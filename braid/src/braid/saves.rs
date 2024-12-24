use crate::{BraidTransaction, Key, Oid, Result, Save, SaveLineageCriteria};

type Lineage = Vec<Save<String, Option<Oid>>>;

pub struct Saves<'b, 't> {
    braid: &'b mut BraidTransaction<'t>,
}

impl<'b, 't> Saves<'b, 't> {
    pub(crate) fn new(braid: &'b mut BraidTransaction<'t>) -> Self {
        Self { braid }
    }

    pub async fn get_lineage<'a, I>(&mut self, branch: Key<'a>, keys: I) -> Result<Lineage>
    where
        I: IntoIterator<Item = Key<'a>>,
    {
        Ok(self
            .braid
            .db
            .get_many(SaveLineageCriteria { branch, keys })
            .await?)
    }
}
