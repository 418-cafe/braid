use std::sync::LazyLock;

use sqlx::{Connection as _, PgConnection, Postgres};

const MISSING_ENV: &str = "The following environment variables must be set to run the tests:
    TEST_DATABASE_HOST
    TEST_DATABASE_PORT
    TEST_DATABASE_ADMIN_USER
    TEST_DATABASE_ADMIN_PASSWORD";

struct Setup {
    host: String,
    port: u16,
    admin_user: String,
    admin_password: String,
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

    fn service_url(&self, service: &str) -> String {
        let Setup { host, port, .. } = self;
        format!("postgres://{service}:{service}@{host}:{port:?}/{service}")
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

    Setup {
        host,
        port,
        admin_user,
        admin_password,
    }
});

pub(crate) struct Connection {
    db_name: String,
    connection: PgConnection,
}

impl Connection {
    pub(crate) async fn begin(&mut self) -> sqlx::Transaction<'_, Postgres> {
        self.connection
            .begin()
            .await
            .expect("Failed to start transaction")
    }

    pub(crate) async fn drop(self) {
        let Self {
            db_name,
            connection,
        } = self;

        connection.close().await.unwrap();

        let mut connection = sqlx::PgConnection::connect(&Setup::get().admin_url())
            .await
            .unwrap();

        sqlx::query(format!("DROP DATABASE \"{}\"", db_name).as_str())
            .execute(&mut connection)
            .await
            .unwrap();
    }
}

async fn setup_user(connection: &mut PgConnection, db_name: &str) {
    sqlx::query(
        format!(
            "
            DO
            $do$
            BEGIN
            IF EXISTS (
                SELECT FROM pg_catalog.pg_roles
                WHERE  rolname = '{db_name}') THEN
            ELSE
                CREATE USER \"{db_name}\" WITH PASSWORD '{db_name}';
            END IF;
            END
            $do$;
        "
        )
        .as_str(),
    )
    .execute(connection)
    .await
    .unwrap();
}

async fn setup_schema(db_name: &str) {
    let url = Setup::get().admin_url();
    let url = format!("{url}/{db_name}");

    let mut connection = sqlx::PgConnection::connect(&url)
        .await
        .expect("Failed to connect to database");

    sqlx::query(format!(r#"CREATE SCHEMA braid_test AUTHORIZATION "{db_name}""#).as_str())
        .execute(&mut connection)
        .await
        .unwrap();

    sqlx::query(format!("ALTER ROLE \"{db_name}\" SET search_path = braid_test, public").as_str())
        .execute(&mut connection)
        .await
        .unwrap();
}

pub(crate) async fn test_database() -> Connection {
    let db_name = uuid::Uuid::new_v4().to_string();

    {
        let mut connection = sqlx::PgConnection::connect(&Setup::get().admin_url())
            .await
            .unwrap();

        sqlx::query(format!("CREATE DATABASE \"{db_name}\"").as_str())
            .execute(&mut connection)
            .await
            .unwrap();

        setup_user(&mut connection, &db_name).await;
    }

    setup_schema(&db_name).await;

    let url = Setup::get().service_url(&db_name);

    println!("Connecting to {}", url);

    let connection = PgConnection::connect(&url).await.unwrap();

    println!("Connected");

    Connection {
        db_name,
        connection,
    }
}
