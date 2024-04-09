use std::{char, fs::{File, ReadDir}};

use hash::{HexByte, HexByteExtensions, HexBytePairExtensions, Oid, OID_LEN};

use crate::{save::SaveData, Object};

mod commit;
mod err;
mod register;
mod save;
mod rw;

type Result<T> = std::result::Result<T, err::Error>;
type HexPairs = [[HexByte; 2]];

type DataSize = u32;

const DATA_SIZE: usize = std::mem::size_of::<DataSize>();
const OBJECT_KIND_SIZE: usize = std::mem::size_of::<crate::ObjectKind>();
const HEADER_SIZE: usize = OBJECT_KIND_SIZE + DATA_SIZE;

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

    pub fn lookup_register(&self, oid: Oid) -> Result<register::ReturnRegisterEntryCollection> {
        let mut file = self.file(oid)?;
        register::read_register(&mut file)
    }

    pub fn lookup_commit(&self, oid: Oid) -> Result<commit::ReturnCommitData> {
        let mut file = self.file(oid)?;
        commit::read(&mut file)
    }

    pub fn lookup_save(&self, oid: Oid) -> Result<SaveData<String>> {
        let mut file = self.file(oid)?;
        save::read(&mut file)
    }

    pub fn lookup_save_register(&self, oid: Oid) -> Result<register::ReturnSaveEntryCollection> {
        let mut file = self.file(oid)?;
        register::read_save_register(&mut file)
    }

    pub fn list(&self) -> Result<ObjectIter> {
        let entries = std::fs::read_dir(&self.mount)?;
        Ok(ObjectIter::new(self, entries))
    }

    fn validate<O: crate::oid::ValidOid>(&self, oid: Oid) -> Result<O> {
        let mut file = self.file(oid)?;
        let mut reader = rw::Reader(&mut file);

        let is_for = reader.read_kind()?;
        if is_for == O::KIND {
            Ok(O::new(oid))
        } else {
            Err(err::Error::InvalidOid { oid, is_for })
        }
    }

    fn file(&self, oid: Oid) -> std::io::Result<File> {
        let path = self.path(oid);
        File::open(path)
    }

    fn path(&self, oid: Oid) -> std::path::PathBuf {
        let pairs = oid.hex_ascii_byte_pairs();
        let (first, second) = segments(&pairs);
        self.mount.join(first).join(second)
    }
}

pub struct ObjectIter<'a> {
    db: &'a Database,
    byte_dirs: ByteDirIter,
    objects: Option<ByteObjIter>,
}

impl<'a> ObjectIter<'a> {
    fn new(db: &'a Database, parent: ReadDir) -> Self {
        let byte_dirs = ByteDirIter { dirs: parent };
        Self { db, byte_dirs, objects: None }
    }

    fn next(&mut self) -> Result<Option<Object>> {
        let mut objects = match self.objects {
            Some(ref mut objects) => objects,
            None => match self.next_objects()? {
                Some(objects) => objects,
                None => return Ok(None),
            }
        };

        loop {
            let object = objects.next();
            if matches!(object, Ok(Some(_)) | Err(_)) {
                break object
            }

            objects = match self.next_objects()? {
                Some(objects) => objects,
                None => break Ok(None),
            }
        }
    }

    fn next_objects(&mut self) -> Result<Option<&mut ByteObjIter>> {
        Ok(match self.byte_dirs.next()? {
            None => None,

            Some((name, byte)) => {
                let dir = self.db.mount.join(name);
                let dir = std::fs::read_dir(dir)?;
                Some(self.objects.insert(ByteObjIter { dir, byte }))
            }
        })
    }
}

impl Iterator for ObjectIter<'_> {
    type Item = Result<Object>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next().transpose()
    }
}

struct ByteDirIter {
    dirs: ReadDir,
}

impl ByteDirIter {
    fn next(&mut self) -> Result<Option<(String, u8)>> {
        Ok(loop {
            let entry = match self.dirs.next() {
                Some(entry) => entry?,
                None => break None,
            };

            let name = if entry.file_type()?.is_dir() {
                entry.file_name().to_string_lossy().into_owned()
            } else {
                continue;
            };

            match byte_from_next_pair(&mut name.chars()) {
                Some(byte) => break Some((name, byte)),
                None => continue,
            };
        })
    }
}

struct ByteObjIter {
    dir: ReadDir,
    byte: u8,
}

impl ByteObjIter {
    fn next(&mut self) -> Result<Option<Object>> {
        Ok('outer: loop{
            let entry = match self.dir.next() {
                Some(entry) => entry?,
                None => break None,
            };

            let name = if entry.file_type()?.is_file() {
                entry.file_name()
            } else {
                continue;
            };

            let mut oid = [0; OID_LEN];
            oid[0] = self.byte;

            let lossy = name.to_string_lossy();
            let mut chars = lossy.chars();
            for i in 1..OID_LEN {
                match byte_from_next_pair(&mut chars) {
                    Some(byte) => oid[i] = byte,
                    None => continue 'outer,
                };
            }

            if chars.next().is_some() {
                continue;
            }

            let oid = Oid::from_bytes(oid);

            let file = File::open(entry.path())?;
            let mut reader = rw::Reader(file);
            let (kind, size) = reader.read_header()?;

            break Some(Object {
                oid,
                kind,
                size,
            })
        })
    }
}

fn byte_from_next_pair(chars: &mut impl Iterator<Item = char>) -> Option<u8> {
    let char = chars.next()?;
    let hi = HexByte::try_from_char(char)?;

    let char = chars.next()?;
    let lo = HexByte::try_from_char(char)?;

    Some(hi.as_hi_with_lo_nibble(lo))
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
    use std::collections::HashMap;

    use hash::Oid;
    use time::{Date, UtcOffset, Weekday::Wednesday};
    use crate::key::Key;

    use crate::{commit::CommitData, register::{RegisterEntryCollection, EntryData, RegisterEntryKind}};

    fn base_register() -> RegisterEntryCollection<&'static str, EntryData> {
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
        let db = super::Database::new(temp.path());

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

        let rehashed = db.write(&register)
            .expect("register should be rehashable");

        assert_eq!(oid, rehashed);

        let register = oid;

        let commit = base_commit(register);
        let oid = db.write(&commit).expect("expected write_commit to succeed");

        let resolved = db.lookup_commit(oid).expect("expected lookup_commit to succeed");

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

        let list: Result<HashMap<_, _>, _> = db
            .list()
            .expect("expected iter to be created")
            .map(|o| o.map(|o| (o.oid, o)))
            .collect();

        let list = list.expect("expected list to be successful");

        const EXPECTED_LEN: usize =
            1 /* register */ +
            1 /* commit */;

        assert_eq!(list.len(), EXPECTED_LEN);
    }
}
