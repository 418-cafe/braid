use crate::Error;

const ERR_DUPLICATE_SCHEMA: &'static str = "42P06";

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        if let Some(err) = err.as_database_error() {
            if let Some(code) = err.code() {
                if let Some(err) = map_postgres_error_code(code.as_ref()) {
                    return err;
                }
            }
        }
        Self::Postgres(err)
    }
}

fn map_postgres_error_code(code: &str) -> Option<crate::Error> {
    use crate::Error::*;
    match code {
        ERR_DUPLICATE_SCHEMA => Some(PostgresBackendAlreadyInitialized),
        _ => None,
    }
}
