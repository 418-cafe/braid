// for now this is just a POC. with packfiles, we'll need to do something more sophisticated

use std::{fs::ReadDir, path::PathBuf};

use hash::Oid;

use super::{commit::ReadCommitData, Database, Result};

struct CommitIter {
    inner: HashFileIter,
}

impl CommitIter {
    pub fn new(db: &Database) -> Result<Self> {
        let dir = db.commits.read_dir()?;
        Ok(Self {
            inner: HashFileIter::new(dir)?,
        })
    }
}

impl Iterator for CommitIter {
    type Item = Result<ReadCommitData>;

    fn next(&mut self) -> Option<Self::Item> {
        let path = match self.inner.next()? {
            Ok(path) => path,
            Err(err) => return Some(Err(err)),
        };
        let mut file = match std::fs::File::open(&path) {
            Ok(file) => file,
            Err(err) => return Some(Err(err.into())),
        };
        Some(super::commit::read(&mut file))
    }
}

struct HashFileIter {
    dir: ReadDir,
}

impl HashFileIter {
    pub fn new(dir: ReadDir) -> Result<Self> {
        Ok(Self { dir })
    }
}

impl Iterator for HashFileIter {
    type Item = Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Ok(loop {
            let file = self.dir.next()?;
            let file = match file {
                Ok(file) => file,
                Err(err) => return Some(Err(err.into())),
            };

            match file.file_type() {
                Ok(ty) if !ty.is_file() => continue,
                Ok(_) => {},
                Err(err) => return Some(Err(err.into())),
            };

            if Oid::try_from_str(file.file_name().to_string_lossy()).is_ok() {
                // todo: probably worth logging the other case
                break file.path()
            }
        }))
    }
}
