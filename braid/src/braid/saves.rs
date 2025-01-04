use crate::{
    db::{self, save::LineageIter},
    FullKey, Result, SaveLineageCriteria,
};

use super::Braid;

pub struct Saves<'b> {
    braid: &'b Braid,
}

impl<'b> Saves<'b> {
    pub(crate) fn new(braid: &'b Braid) -> Self {
        Self { braid }
    }

    pub async fn get_lineage<'a, I>(
        &mut self,
        branch: FullKey<&'a str>,
        keys: I,
    ) -> Result<LineageIter>
    where
        I: IntoIterator<Item = FullKey<&'a str>>,
    {
        Ok(db::save::lineage(&self.braid.pool, SaveLineageCriteria { branch, keys }).await?)
    }
}
