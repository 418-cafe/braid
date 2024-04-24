use crate::{postgres::Transaction, Result};

pub(super) async fn init(tran: &mut Transaction<'_>) -> Result<()> {
    Ok(())
}

async fn init_refs() {
    const CREATE: &str = "
        CREATE TABLE IF NOT EXISTS refs (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            hash TEXT NOT NULL
        );
        ";
}
