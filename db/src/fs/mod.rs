use std::fs::File;

use hash::Oid;

use crate::{save::SaveData, ObjectKind};

mod commit;
mod err;
mod iter;
mod register;
mod rw;
mod save;

type Result<T>  = std::result::Result<T, err::Error>;

type DataSize = u32;

const DATA_SIZE: usize = std::mem::size_of::<DataSize>();
const OBJECT_KIND_SIZE: usize = std::mem::size_of::<crate::ObjectKind>();
const HEADER_SIZE: usize = OBJECT_KIND_SIZE + DATA_SIZE;

pub struct Database {
    pub(crate) mount: std::path::PathBuf,
    pub(crate) commits: std::path::PathBuf,
    pub(crate) registers: std::path::PathBuf,
    pub(crate) save_registers: std::path::PathBuf,
    pub(crate) saves: std::path::PathBuf,
}

impl Database {
    pub fn init(mount: impl Into<std::path::PathBuf>) -> Result<Self> {
        let mount = mount.into();
        let commits = mount.join(ObjectKind::Commit.dir());
        let registers = mount.join(ObjectKind::Register.dir());
        let save_registers = mount.join(ObjectKind::SaveRegister.dir());
        let saves = mount.join(ObjectKind::Save.dir());

        for dir in [&commits, &registers, &save_registers, &saves] {
            std::fs::create_dir_all(dir)?;
        }

        Ok(Self {
            mount,
            commits,
            registers,
            save_registers,
            saves,
        })
    }

    pub fn mount(&self) -> &std::path::Path {
        &self.mount
    }

    pub fn hash(&self, object: impl Hash) -> Result<Oid> {
        object.hash().map(|(oid, _)| oid)
    }

    pub fn write<H: Hash + Validate>(&self, object: &H) -> Result<Oid> {
        object.validate(self)?;

        let (oid, data) = object.hash()?;
        let path = self.dir(H::KIND).join(oid.to_hex_string());
        std::fs::write(path, data)?;

        Ok(oid)
    }

    pub fn lookup_register(&self, oid: Oid) -> Result<register::ReadRegisterEntryCollection> {
        let file = self.registers.join(oid.to_hex_string());
        let mut file = File::open(file)?;
        register::read_register(&mut file)
    }

    pub fn lookup_commit(&self, oid: Oid) -> Result<commit::ReadCommitData> {
        let file = self.commits.join(oid.to_hex_string());
        let mut file = File::open(file)?;
        commit::read(&mut file)
    }

    pub fn lookup_save(&self, oid: Oid) -> Result<SaveData<String>> {
        let file = self.saves.join(oid.to_hex_string());
        let mut file = File::open(file)?;
        save::read(&mut file)
    }

    pub fn lookup_save_register(&self, oid: Oid) -> Result<register::ReadSaveEntryCollection> {
        let file = self.save_registers.join(oid.to_hex_string());
        let mut file = File::open(file)?;
        register::read_save_register(&mut file)
    }

    fn try_validate<O: crate::oid::ValidOid>(&self, oid: Oid) -> Result<O> {
        let dir = self.dir(O::KIND);
        let path = dir.join(oid.to_hex_string());
        path.exists()
            .then_some(O::new(oid))
            .ok_or(err::Error::ObjectNotFound(O::KIND, oid))
    }

    const fn dir(&self, kind: ObjectKind) -> &std::path::PathBuf {
        match kind {
            ObjectKind::Commit => &self.commits,
            ObjectKind::Register => &self.registers,
            ObjectKind::SaveRegister => &self.save_registers,
            ObjectKind::Save => &self.saves,
        }
    }
}

impl ObjectKind {
    const fn dir(&self) -> &'static str {
        match self {
            Self::Commit => "commits",
            Self::Register => "registers",
            Self::SaveRegister => "save_registers",
            Self::Save => "saves",
        }
    }
}

pub trait Hash: crate::sealed::Sealed {
    const KIND: ObjectKind;

    fn hash(&self) -> Result<(hash::Oid, Vec<u8>)>;
}

pub trait Validate: crate::sealed::Sealed {
    fn validate(&self, _db: &Database) -> Result<()> {
        Ok(())
    }
}

impl<T: Hash> Hash for &T {
    const KIND: ObjectKind = T::KIND;

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
    use crate::key::Key;
    use hash::Oid;
    use time::{Date, UtcOffset, Weekday::Wednesday};

    use crate::{
        commit::CommitData,
        register::{EntryData, RegisterEntryCollection, RegisterEntryKind},
    };

    fn base_register() -> RegisterEntryCollection<&'static str, EntryData> {
        [
            (
                Key::try_from("foo").unwrap(),
                EntryData::new(RegisterEntryKind::Content, Oid::repeat(1)),
            ),
            (
                Key::try_from("bar").unwrap(),
                EntryData::new(RegisterEntryKind::Register, Oid::repeat(2)),
            ),
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
            saves: Oid::repeat(4),
            date,
            committer: "bruce@wayne.ent",
            summary: "This is a summary",
            body: "This is a body\nwith multiple lines\nand a trailing newline\n",
        }
    }

    #[test]
    fn test_rehashibility() {
        let temp = tempdir::TempDir::new("db").unwrap();
        let db = super::Database::init(temp.path()).expect("expected init to succeed");

        let register = base_register();
        let oid = db.write(&register).expect("expected hash to succeed");

        let register = db
            .lookup_register(oid)
            .expect("expected lookup_register to succeed");

        assert_eq!(register.len(), 2);

        // should be sorted by name
        let entry = register.get("bar").expect("expected entry to exist");
        assert_eq!(entry.kind, RegisterEntryKind::Register);
        assert_eq!(entry.content, Oid::repeat(2));

        let entry = register.get("foo").expect("expected entry to exist");
        assert_eq!(entry.kind, RegisterEntryKind::Content);

        let rehashed = db.write(&register).expect("register should be rehashable");

        assert_eq!(oid, rehashed);

        let register = oid;

        let commit = base_commit(register);
        let oid = db.write(&commit).expect("expected write_commit to succeed");

        let resolved = db
            .lookup_commit(oid)
            .expect("expected lookup_commit to succeed");

        assert_eq!(resolved.register, commit.register);
        assert_eq!(resolved.parent, commit.parent);
        assert_eq!(resolved.merge_parent, commit.merge_parent);
        assert_eq!(resolved.rebase_of, commit.rebase_of);
        // todo: assert
        // assert_eq!(resolved.saves, commit.saves);
        assert_eq!(resolved.date, commit.date);
        assert_eq!(resolved.committer, commit.committer);
        assert_eq!(resolved.summary, commit.summary);
        assert_eq!(resolved.body, commit.body);
    }
}
