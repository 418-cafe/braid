use braid_hash::Oid;

use crate::{
    bytes::Hash,
    postgres::Executor,
    save::{Save, SaveData},
    Result,
};

use super::OidData;

pub(super) async fn get(oid: Oid, exec: impl Executor<'_>) -> Result<Option<Save<String>>> {
    let save = sqlx::query_as("SELECT * FROM braid.get_save($1)")
        .bind(oid)
        .fetch_optional(exec)
        .await?;
    Ok(save)
}

impl<S: AsRef<str>> super::write::Write for SaveData<S> {
    async fn write(&self, exec: impl super::Executor<'_>) -> Result<Oid> {
        let (id, _) = Hash::hash(self)?;

        sqlx::query("CALL braid.create_save($1, $2::varchar, $3, $4, $5)")
            .bind(id)
            .bind(self.author.as_ref())
            .bind(self.date)
            .bind(self.content)
            .bind(self.parent)
            .execute(exec)
            .await?;

        Ok(id)
    }
}

impl OidData<'_> for Save<String> {
    type Data = SaveData<String>;

    fn create(id: Oid, data: Self::Data) -> Self {
        Self { id, data }
    }
}
