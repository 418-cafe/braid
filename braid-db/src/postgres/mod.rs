use braid_hash::Oid;
use sqlx::Postgres;

use crate::Result;

mod commit;
mod init;
mod register;
mod save;
mod save_register;

type Transaction<'a> = sqlx::Transaction<'a, Postgres>;
pub trait Executor<'a>: sqlx::Executor<'a, Database = Postgres> {}
impl<'a, T: sqlx::Executor<'a, Database = Postgres>> Executor<'a> for T {}

pub async fn init(db: &sqlx::PgPool) -> Result<()> {
    let mut tran = db.begin().await?;
    init::init(&mut tran).await?;
    tran.commit().await?;
    Ok(())
}

pub async fn get_commit(
    oid: Oid,
    exec: impl Executor<'_>,
) -> Result<Option<crate::commit::Commit>> {
    commit::get(oid, exec).await
}

pub async fn write(obj: &impl write::Write, exec: impl Executor<'_>) -> Result<Oid> {
    obj.write(exec).await
}

pub async fn write_to_tran(
    obj: &impl write_to_tran::Write,
    tran: &mut Transaction<'_>,
) -> Result<Oid> {
    obj.write(tran).await
}

pub async fn register_as_content(oid: Oid, exec: impl Executor<'_>) -> Result<()> {
    sqlx::query("INSERT INTO braid_obj.content_register (id, is_content) VALUES ($1, true)")
        .bind(oid.as_bytes())
        .execute(exec)
        .await?;
    Ok(())
}

// this is in an inernal trait and just a way to genericize the `write_to_tran` function
#[allow(async_fn_in_trait)]
mod write {
    pub trait Write {
        async fn write(&self, exec: impl super::Executor<'_>) -> crate::Result<braid_hash::Oid>;
    }
}

// this is in an inernal module and just a way to genericize the `write` function
#[allow(async_fn_in_trait)]
mod write_to_tran {
    pub trait Write {
        async fn write(&self, tran: &mut super::Transaction<'_>) -> crate::Result<braid_hash::Oid>;
    }

    impl<T: super::write::Write> Write for T {
        #[inline]
        async fn write(&self, tx: &mut super::Transaction<'_>) -> crate::Result<braid_hash::Oid> {
            <Self as super::write::Write>::write(self, &mut **tx).await
        }
    }
}
