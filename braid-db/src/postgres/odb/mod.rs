use braid_hash::Oid;
use sqlx::postgres::PgRow;

use crate::{commit::Commit, register::SaveRegister, save::Save, Result};

use super::Executor;

mod commit;
mod key;
mod register;
mod save;
mod save_register;

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

pub async fn create_content(oid: Oid, exec: impl Executor<'_>) -> Result<()> {
    sqlx::query("CALL braid.create_content($1)")
        .bind(oid)
        .execute(exec)
        .await?;
    Ok(())
}

pub async fn write(obj: &impl write::Write, exec: impl Executor<'_>) -> Result<Oid> {
    obj.write(exec).await
}

// this is in an internal trait and just a way to genericize the `write_to_tran` function
#[allow(async_fn_in_trait)]
mod write {
    pub trait Write {
        async fn write(&self, exec: impl super::Executor<'_>) -> crate::Result<braid_hash::Oid>;
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
