use crate::data::Commit;

use super::{Braid, Error, Result};

pub struct Commits<'b, 't> {
    braid: &'b mut Braid<'b, 't>,
}

impl<'b, 't> Commits<'b, 't> {
    pub(crate) fn new(braid: &'b mut Braid<'b, 't>) -> Self {
        Self { braid }
    }

    pub async fn get_root(&mut self) -> Result<Commit> {
        self.braid
            .db
            .get_root()
            .await?
            .ok_or(const { Error::RootCommitDoesNotExist })
    }
}
