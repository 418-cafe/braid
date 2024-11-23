use sqlx::PgPool;

pub struct Database {
    pool: PgPool,
}