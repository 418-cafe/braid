use sqlx::PgPool;

use crate::{
    db::{Database, Error},
    object::Object, Oid,
};

pub struct Braid {
    db: Database,
}

impl Braid {
    pub async fn init_with_pool(pool: PgPool) -> Result<Self, Error> {
        let db = Database::with_pool(pool);
        Ok(Braid { db })
    }

    pub async fn with_pool(pool: PgPool) -> Self {
        let db = Database::with_pool(pool);
        Braid { db }
    }

    pub async fn get<T: Object>(&self, oid: Oid) -> Result<Option<T>, Error> {
        self.db.get(oid).await
    }
}