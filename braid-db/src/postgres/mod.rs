use sqlx::{PgPool, Postgres};

use crate::Result;

mod err;
mod init;

pub mod odb;
pub mod state;

type Transaction<'a> = sqlx::Transaction<'a, Postgres>;
pub trait Executor<'a>: sqlx::Executor<'a, Database = Postgres> {}
impl<'a, T: sqlx::Executor<'a, Database = Postgres>> Executor<'a> for T {}

pub async fn init(pool: &PgPool) -> Result<()> {
    init::init(pool).await
}
