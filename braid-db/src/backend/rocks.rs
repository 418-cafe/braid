use braid_hash::Oid;
use rocksdb::DBWithThreadMode;

use crate::{
    bytes::{commit, register, save, Hash},
    err::Error,
    ObjectKind, Result,
};

// single threading refers to how column families are added/removed.
// we don't use column families, so we can use single threaded mode.
type Db = DBWithThreadMode<rocksdb::SingleThreaded>;

pub struct Database {
    pub(crate) mount: std::path::PathBuf,
    pub(crate) db: Db,
}

impl Database {
    pub fn init(mount: impl Into<std::path::PathBuf>) -> Result<Self> {
        let mount = mount.into();
        let db = rocksdb::DB::open_default(&mount)?;
        Ok(Self { mount, db })
    }

    pub fn mount(&self) -> &std::path::Path {
        &self.mount
    }

    pub fn write<H: Hash>(&self, object: &H) -> Result<Oid> {
        let (oid, data) = object.hash()?;
        self.db.put(oid.as_bytes(), &data)?;
        Ok(oid)
    }

    pub fn lookup_register(&self, oid: Oid) -> Result<register::ReadRegisterEntryCollection> {
        let mut reader = self.get_reader(ObjectKind::Register, oid)?;
        register::read_register(&mut reader)
    }

    pub fn lookup_commit(&self, oid: Oid) -> Result<commit::ReadCommitData> {
        let mut reader = self.get_reader(ObjectKind::Commit, oid)?;
        commit::read(&mut reader)
    }

    pub fn lookup_save(&self, oid: Oid) -> Result<save::ReadSaveData> {
        let mut reader = self.get_reader(ObjectKind::Save, oid)?;
        save::read(&mut reader)
    }

    pub fn lookup_save_register(&self, oid: Oid) -> Result<register::ReadSaveEntryCollection> {
        let mut reader = self.get_reader(ObjectKind::SaveRegister, oid)?;
        register::read_save_register(&mut reader)
    }

    fn get(
        &self,
        kind: ObjectKind,
        oid: Oid,
    ) -> std::result::Result<rocksdb::DBPinnableSlice<'_>, Error> {
        let data = self.db.get_pinned(oid.as_bytes())?;
        data.ok_or_else(move || Error::ObjectNotFound(kind, oid))
    }

    fn get_reader(&self, kind: ObjectKind, oid: Oid) -> Result<impl '_ + std::io::Read> {
        let data = self.get(kind, oid)?;
        Ok(std::io::Cursor::new(data))
    }
}

#[cfg(test)]
mod tests {
    use crate::key::Key;
    use braid_hash::Oid;
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
