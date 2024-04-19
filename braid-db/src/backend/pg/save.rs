use crate::{bytes::Hash, save::SaveData, Result};

impl<S: AsRef<str>> super::queryable::QueryAble for SaveData<S> {
    fn insert_query(&self) -> Result<super::Query> {
        let (id, _) = Hash::hash(self)?;

        let query = sqlx::query(
            "
            INSERT INTO obj.save (id, author, date, kind, content, parent)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT DO NOTHING
            ",
        )
        .bind(*id.as_bytes())
        .bind(self.author.as_ref())
        .bind(self.date)
        .bind(self.kind as i16)
        .bind(self.content.as_bytes())
        .bind(self.parent.oid.as_bytes());

        Ok(query)
    }
}
