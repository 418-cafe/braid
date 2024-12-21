use crate::{BraidTransaction, CommitWithImpl, Error, Result};

pub struct Commits<'b, 't> {
    braid: &'b mut BraidTransaction<'t>,
}

impl<'b, 't> Commits<'b, 't> {
    pub(crate) fn new(braid: &'b mut BraidTransaction<'t>) -> Self {
        Self { braid }
    }

    pub async fn get_root(&mut self) -> Result<CommitWithImpl> {
        self.braid
            .db
            .get_root()
            .await?
            .ok_or(const { Error::RootCommitDoesNotExist })
    }
}
