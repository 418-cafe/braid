use sqlx::PgPool;

use crate::{
    commit::Commit,
    register::{Register, SaveRegister},
    Result,
};

pub(super) async fn init(pool: &PgPool) -> Result<()> {
    let mut tran = pool.begin().await?;

    sqlx::query("CREATE SCHEMA braid;")
        .execute(&mut *tran)
        .await?;
    sqlx::query(include_str!("sql/init-braid.sql"))
        .execute(&mut *tran)
        .await?;
    sqlx::query("CALL braid.init_braid();")
        .execute(&mut *tran)
        .await?;
    sqlx::query("DROP PROCEDURE braid.init_braid;")
        .execute(&mut *tran)
        .await?;

    sqlx::query("CALL braid.create_register($1, ARRAY[]::braid.register_entry_records)")
        .bind(Register::EMPTY_ID)
        .execute(&mut *tran)
        .await?;

    sqlx::query("CALL braid.create_save_register($1, ARRAY[]::braid.save_register_entry_records)")
        .bind(SaveRegister::EMPTY_ID)
        .execute(&mut *tran)
        .await?;

    super::odb::write(&Commit::ROOT.data, &mut *tran).await?;

    tran.commit().await?;
    Ok(())
}
