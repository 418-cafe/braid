use std::{fs::File, io::Read};

use hash::{HexByte, HexByteExtensions, HexBytePairExtensions, Oid};

use crate::{commit::CommitData, save::SaveData, Kind, Object, ObjectKind};

mod commit;
mod err;
mod register;
mod save;
mod rw;

type Result<T> = std::result::Result<T, err::Error>;
type HexPairs = [[HexByte; 2]];

// todo: abstract over a packfile?
type Location = String;

pub struct Database {
    pub(crate) mount: std::path::PathBuf,
}

impl Database {
    pub fn new(mount: impl Into<std::path::PathBuf>) -> Self {
        Self {
            mount: mount.into(),
        }
    }

    pub fn hash(&self, object: impl Hash) -> Result<Oid> {
        object.hash().map(|(oid, _)| oid)
    }

    pub fn write(&self, object: &(impl Hash + Validate)) -> Result<Oid> {
        object.validate(self)?;

        let (oid, data) = object.hash()?;
        let pairs = oid.hex_ascii_byte_pairs();

        let (first, second) = segments(&pairs);
        let path = self.mount.join(first);

        std::fs::create_dir_all(path.clone())?;
        assert!(path.as_path().exists());
        std::fs::write(path.join(second), data)?;

        Ok(oid)
    }

    pub fn lookup<'a>(&self, oid: Oid) -> Option<Object<Location>> {
        let pairs = oid.hex_ascii_byte_pairs();

        let (first, second) = segments(&pairs);

        let path = self.mount.join(first).join(second);
        let location = path.to_string_lossy().to_string();

        let mut file = std::fs::File::open(&path).ok()?;
        let mut buf = [0; 1];
        file.read_exact(&mut buf).ok()?;

        let kind = ObjectKind::from_u8(buf[0])?;

        Some(Object { kind, location })
    }

    pub fn lookup_register(&self, oid: Oid) -> Result<register::RegisterReadIter<File>> {
        let path = self.path(oid);
        let file = File::open(path)?;
        register::read(file)
    }

    pub fn lookup_commit(&self, oid: Oid) -> Result<CommitData<String>> {
        let path = self.path(oid);
        let file = File::open(path)?;
        let commit = commit::read(file)?;
        commit.validate(self)?;
        Ok(commit)
    }

    pub fn lookup_save(&self, oid: Oid) -> Result<SaveData<String>> {
        todo!()
        // let path = self.path(oid);
        // let file = File::open(path)?;
        // let save = save::read(file)?;
        // save.validate(self)?;
        // Ok(save)
    }

    fn path(&self, oid: Oid) -> std::path::PathBuf {
        let pairs = oid.hex_ascii_byte_pairs();
        let (first, second) = segments(&pairs);
        self.mount.join(first).join(second)
    }
}

fn segments(pairs: &HexPairs) -> (&str, &str) {
    let first = pairs[0].as_str();
    let second = pairs[1..].flat().as_str();
    (first, second)
}

pub trait Hash: crate::sealed::Sealed {
    fn hash(&self) -> Result<(hash::Oid, Vec<u8>)>;
}

pub trait Validate: crate::sealed::Sealed {
    fn validate(&self, _db: &Database) -> Result<()> {
        Ok(())
    }
}

impl<T: Hash> Hash for &T {
    fn hash(&self) -> Result<(hash::Oid, Vec<u8>)> {
        (*self).hash()
    }
}

impl<T: Validate> Validate for &T {
    fn validate(&self, db: &Database) -> Result<()> {
        (*self).validate(db)
    }
}

#[cfg(test)]
mod tests {
    use hash::Oid;
    use time::{Date, UtcOffset, Weekday::Wednesday};
    use crate::key::Key;

    use crate::{
        commit::CommitData, register::{RegisterEntryCollection, EntryData, RegisterEntryKind}, save::SaveData, ObjectKind
    };

    fn base_register() -> RegisterEntryCollection<Key<&'static str>, EntryData<RegisterEntryKind>> {
        [
            (Key::try_from("foo").unwrap(), EntryData::new(RegisterEntryKind::Content, Oid::repeat(1))),
            (Key::try_from("bar").unwrap(), EntryData::new(RegisterEntryKind::Register, Oid::repeat(2))),
        ]
        .into_iter()
        .collect()
    }

    // fn base_save() -> SaveData<&'static str> {
    //     let date = Date::from_iso_week_date(2022, 1, Wednesday).unwrap();
    //     let date = date.with_hms(13, 0, 55).unwrap();
    //     let date = date.assume_offset(UtcOffset::from_hms(1, 2, 3).unwrap());

    //     SaveData {
    //         date,
    //         content: Oid::repeat(3),
    //         author: "bruce@wayne.ent",
    //     }
    // }

    fn base_commit(register: Oid) -> CommitData<&'static str> {
        let date = Date::from_iso_week_date(2022, 1, Wednesday).unwrap();
        let date = date.with_hms(13, 0, 55).unwrap();
        let date = date.assume_offset(UtcOffset::from_hms(1, 2, 3).unwrap());

        CommitData {
            register,
            parent: None,
            merge_parent: None,
            rebase_of: None,
            saves: vec![],
            date,
            committer: "bruce@wayne.ent",
            summary: "This is a summary",
            body: "This is a body\nwith multiple lines\nand a trailing newline\n",
        }
    }

    #[test]
    fn test_rehashibility() {
        let temp = tempdir::TempDir::new("db").unwrap();
        let db = super::Database::new(temp.path());

        let register = base_register();
        let oid = db.write(&register).expect("expected hash to succeed");

        let object = db.lookup(oid).expect("expected lookup to succeed");
        assert_eq!(object.kind, ObjectKind::Register);

        let register: Result<Vec<_>, _> = db
            .lookup_register(oid)
            .expect("expected lookup_register to succeed")
            .map(|r| r.map(|(name, entry)| (Key::try_from(name).unwrap(), entry)))
            .collect();

        let register = register.expect("expected register to be read");

        assert_eq!(register.len(), 2);

        // should be sorted by name
        let (name, entry) = &register[0];
        assert_eq!(name.as_ref(), "bar");
        assert_eq!(entry.kind, RegisterEntryKind::Register);

        let (name, entry) = &register[1];
        assert_eq!(name.as_ref(), "foo");
        assert_eq!(entry.kind, RegisterEntryKind::Content);

        let rehashed = db.write(&RegisterEntryCollection::from_iter(register.iter()))
            .expect("register should be rehashable");

        assert_eq!(oid, rehashed);

        let register = oid;

        let commit = base_commit(register);
        let oid = db.write(&commit).expect("expected write_commit to succeed");

        let object = db.lookup(oid).expect("expected lookup to succeed");
        assert_eq!(object.kind, ObjectKind::Commit);

        let resolved = db.lookup_commit(oid).expect("expected lookup_commit to succeed");

        assert_eq!(resolved.register, commit.register);
        assert_eq!(resolved.parent, commit.parent);
        assert_eq!(resolved.merge_parent, commit.merge_parent);
        assert_eq!(resolved.rebase_of, commit.rebase_of);
        assert_eq!(resolved.saves, commit.saves);
        assert_eq!(resolved.date, commit.date);
        assert_eq!(resolved.committer, commit.committer);
        assert_eq!(resolved.summary, commit.summary);
        assert_eq!(resolved.body, commit.body);
    }
}
