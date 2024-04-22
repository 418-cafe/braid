use braid_hash::Oid;
use sqlx::{postgres::PgRow, Postgres};

use crate::{commit::Commit, register::SaveRegister, save::Save, Result};

mod commit;
mod init;
mod register;
mod save;
mod save_register;

type Transaction<'a> = sqlx::Transaction<'a, Postgres>;
pub trait Executor<'a>: sqlx::Executor<'a, Database = Postgres> {}
impl<'a, T: sqlx::Executor<'a, Database = Postgres>> Executor<'a> for T {}

pub async fn get_commit(
    oid: Oid,
    exec: impl Executor<'_>,
) -> Result<Option<crate::commit::Commit>> {
    commit::get(oid, exec).await
}

pub async fn get_register(
    oid: Oid,
    exec: impl Executor<'_>,
) -> Result<Option<crate::register::Register>> {
    register::get(oid, exec).await
}

pub async fn get_save(oid: Oid, exec: impl Executor<'_>) -> Result<Option<crate::save::Save>> {
    save::get(oid, exec).await
}

pub async fn get_save_register(
    oid: Oid,
    exec: impl Executor<'_>,
) -> Result<Option<SaveRegister<String>>> {
    save_register::get(oid, exec).await
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
    sqlx::query("INSERT INTO braid.content_register (id, is_content) VALUES ($1, true)")
        .bind(oid.as_bytes())
        .execute(exec)
        .await?;
    Ok(())
}

pub(super) async fn init(tran: &mut Transaction<'_>) -> Result<()> {
    init::init(tran).await
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

trait OidData<'a>: Sized {
    type Data: sqlx::FromRow<'a, PgRow>;

    fn create(id: Oid, data: Self::Data) -> Self;

    fn from_row(row: &'a PgRow) -> std::result::Result<Self, sqlx::Error> {
        use sqlx::{FromRow, Row};

        let id = row.try_get("id")?;
        let data = Self::Data::from_row(row)?;
        Ok(Self::create(id, data))
    }
}

macro_rules! impl_from_row {
    ($($t:ty),* $(,)?) => {
        $(
            impl<'a> sqlx::FromRow<'a, PgRow> for $t {
                fn from_row(row: &'a PgRow) -> std::prelude::v1::Result<Self, sqlx::Error> {
                    <Self as OidData<'a>>::from_row(row)
                }
            }
        )*
    };
}

impl_from_row!(Commit<String>, Save<String>);
