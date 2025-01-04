use crate::{db, CommitWithImpl, Error, Result};

use super::Braid;

pub struct Commits<'b> {
    braid: &'b Braid,
}

impl<'b> Commits<'b> {
    pub(crate) fn new(braid: &'b Braid) -> Self {
        Self { braid }
    }

    pub async fn get_root(&mut self) -> Result<CommitWithImpl> {
        db::commit::get_root(&self.braid.pool)
            .await?
            .ok_or(const { Error::RootCommitDoesNotExist })
    }
}
