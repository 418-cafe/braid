use std::sync::LazyLock;

use sqlx::{Connection as _, PgConnection, Pool, Postgres};

const MISSING_ENV: &str = "The following environment variables must be set to run the tests:
    TEST_DATABASE_HOST
    TEST_DATABASE_PORT
    TEST_DATABASE_ADMIN_USER
    TEST_DATABASE_ADMIN_PASSWORD
    TEST_DATABASE_SERVICE_USER
    TEST_DATABASE_SERVICE_PASSWORD";

struct Setup {
    host: String,
    port: u16,
    admin_user: String,
    admin_password: String,
    service_user: String,
    service_password: String,
    pool: Pool<Postgres>,
}

impl Setup {
    fn get() -> &'static Self {
        &*SETUP
    }

    fn admin_url(&self) -> String {
        let Setup {
            host,
            port,
            admin_user,
            admin_password,
            ..
        } = self;
        format!("postgres://{admin_user}:{admin_password}@{host}:{port:?}")
    }

    fn service_url(&self) -> String {
        let Setup {
            host,
            port,
            service_user,
            service_password,
            ..
        } = self;
        format!("postgres://{service_user}:{service_password}@{host}:{port:?}")
    }
}

static SETUP: LazyLock<Setup> = LazyLock::new(|| {
    let _ = dotenvy::dotenv();
    let env = |name| std::env::var(name).expect(MISSING_ENV);

    let host = env("TEST_DATABASE_HOST");
    let port = env("TEST_DATABASE_PORT")
        .parse()
        .expect("TEST_DATABASE_PORT must be a number");
    let admin_user = env("TEST_DATABASE_ADMIN_USER");
    let admin_password = env("TEST_DATABASE_ADMIN_PASSWORD");
    let service_user = env("TEST_DATABASE_SERVICE_USER");
    let service_password = env("TEST_DATABASE_SERVICE_PASSWORD");

    let url = format!("postgres://{admin_user}:{admin_password}@{host}:{port:?}");

    let pool = sqlx::PgPool::connect_lazy(&url).unwrap();

    Setup {
        host,
        port,
        admin_user,
        admin_password,
        service_user,
        service_password,
        pool,
    }
});

pub(crate) struct Transaction<'t>(sqlx::Transaction<'t, Postgres>);

impl<'t> Transaction<'t> {
    pub(crate) fn get_mut(&mut self) -> &mut sqlx::Transaction<'t, Postgres> {
        &mut self.0
    }

    pub(crate) async fn rollback(self) {
        self.0
            .rollback()
            .await
            .expect("Failed to rollback transaction");
    }

    pub(crate) async fn commit(self) {
        self.0.commit().await.expect("Failed to commit transaction");
    }
}

pub(crate) struct Connection {
    db_name: String,
    connection: PgConnection,
}

impl Connection {
    pub(crate) async fn begin(&mut self) -> Transaction<'_> {
        let tx = self
            .connection
            .begin()
            .await
            .expect("Failed to start transaction");
        Transaction(tx)
    }

    pub(crate) async fn drop(self) {
        let Self {
            db_name,
            connection,
        } = self;

        connection.close().await.unwrap();

        sqlx::query(format!("DROP DATABASE \"{}\"", db_name).as_str())
            .execute(&Setup::get().pool)
            .await
            .unwrap();
    }
}

async fn setup_user() {
    let user_exists = sqlx::query("SELECT 1 FROM pg_roles WHERE rolname = $1")
        .bind(&Setup::get().service_user)
        .fetch_optional(&Setup::get().pool)
        .await
        .unwrap()
        .is_some();

    if user_exists {
        return;
    }

    let Setup {
        service_user,
        service_password,
        pool,
        ..
    } = Setup::get();

    sqlx::query(
        format!("CREATE USER \"{service_user}\" WITH PASSWORD '{service_password}'").as_str(),
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn setup_schema(db_name: &str) {
    let url = Setup::get().admin_url();
    let url = format!("{url}/{db_name}");

    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    let Setup { service_user, .. } = Setup::get();

    sqlx::query(format!(r#"CREATE SCHEMA braid_test AUTHORIZATION "{service_user}""#).as_str())
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        format!("ALTER ROLE \"{service_user}\" SET search_path = braid_test, public").as_str(),
    )
    .execute(&pool)
    .await
    .unwrap();
}

pub(crate) async fn test_database() -> Connection {
    let db_name = uuid::Uuid::new_v4().to_string();

    sqlx::query(format!("CREATE DATABASE \"{db_name}\"").as_str())
        .execute(&Setup::get().pool)
        .await
        .unwrap();

    setup_user().await;
    setup_schema(&db_name).await;

    let url = Setup::get().service_url();
    let url = format!("{url}/{db_name}");

    println!("Connecting to {}", url);

    let connection = PgConnection::connect(&url).await.unwrap();

    println!("Connected");

    Connection {
        db_name,
        connection,
    }
}
