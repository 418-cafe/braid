use super::Transaction;
use crate::{
    commit::{Commit, CommitData},
    err::Error,
    register::{Register, RegisterEntryKind, SaveRegister},
    save::SaveParentKind,
    Result,
};

type Query<'a> = sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>;

const ERR_DUPLICATE_SCHEMA: &str = "42P06";
const OID_LEN_STR: &'static str = const_format::concatcp!(braid_hash::Oid::LEN);

pub(super) async fn init(tran: &mut Transaction<'_>) -> Result<()> {
    init_schema(tran).await?;
    init_content_register(tran).await?;
    init_save_parent(tran).await?;
    init_save(tran).await?;
    init_save_register(tran).await?;
    init_save_register_entry(tran).await?;
    init_register_entry(tran).await?;
    init_commit(tran).await?;

    insert_base_data(tran).await
}

async fn init_schema(tran: &mut Transaction<'_>) -> Result<()> {
    const CREATE: &str = "CREATE SCHEMA braid_obj;";

    sqlx::query(CREATE)
        .execute(&mut **tran)
        .await
        .map_err(|e| match e.as_database_error() {
            Some(e)
                if match e.code() {
                    Some(code) => code == ERR_DUPLICATE_SCHEMA,
                    None => false,
                } =>
            {
                Error::PostgresBackendAlreadyInitialized
            }
            _ => e.into(),
        })
        .map(|_| ())
}

async fn init_content_register(tran: &mut Transaction<'_>) -> Result<()> {
    // the content table is not used only for actual content, but also registers
    // so that entries can have an FK to it
    const CREATE: &str = const_format::formatcp!(
        "
        CREATE TABLE braid_obj.content_register (
            id bytea PRIMARY KEY,
            is_content boolean NOT NULL,

            CHECK (octet_length(id) = {0})
        );
        ",
        OID_LEN_STR,
    );

    const CREATE_CONTENT: &str = "
        CREATE TABLE braid_obj.content (
            id bytea PRIMARY KEY,

            FOREIGN KEY (id) REFERENCES braid_obj.content_register (id)
        );
        ";

    const CREATE_REGISTER: &str = "
        CREATE TABLE braid_obj.register (
            id bytea PRIMARY KEY,

            FOREIGN KEY (id) REFERENCES braid_obj.content_register (id)
        );
        ";

    const CREATE_FUNC: &str = "
        CREATE FUNCTION braid_obj.content_register_propagate() RETURNS TRIGGER AS $$
        BEGIN
            IF NEW.is_content THEN
                INSERT INTO braid_obj.content (id) VALUES (NEW.id);
            ELSE
                INSERT INTO braid_obj.register (id) VALUES (NEW.id);
            END IF;
            RETURN NEW;
        END; $$ LANGUAGE plpgsql;
        ";

    const CREATE_TRIGGER: &str = "
        CREATE TRIGGER trigger_content_register_propagate
        AFTER INSERT ON braid_obj.content_register
        FOR EACH ROW EXECUTE FUNCTION braid_obj.content_register_propagate();
        ";

    run_str(tran, CREATE).await?;
    run_str(tran, CREATE_CONTENT).await?;
    run_str(tran, CREATE_REGISTER).await?;
    run_str(tran, CREATE_FUNC).await?;
    run_str(tran, CREATE_TRIGGER).await?;

    Ok(())
}

async fn init_save_parent(tran: &mut Transaction<'_>) -> Result<()> {
    const CREATE: &str = const_format::formatcp!(
        "
        CREATE TABLE braid_obj.save_parent (
            id bytea PRIMARY KEY,
            is_commit boolean NOT NULL,

            CHECK (octet_length(id) = {0})
        );
        ",
        OID_LEN_STR,
    );

    run_str(tran, CREATE).await
}

async fn init_save(tran: &mut Transaction<'_>) -> Result<()> {
    const CREATE: &str = const_format::formatcp!(
        "
            CREATE TABLE braid_obj.save (
                id bytea PRIMARY KEY,
                author varchar(255) NOT NULL,
                date timestamp with time zone NOT NULL,
                kind smallint NOT NULL,
                content bytea NOT NULL,
                parent bytea NOT NULL,

                FOREIGN KEY (id) REFERENCES braid_obj.save_parent (id),
                FOREIGN KEY (parent) REFERENCES braid_obj.save_parent (id),
                FOREIGN KEY (content) REFERENCES braid_obj.content (id),

                CHECK (kind IN ({0}))
            );
        ",
        RegisterEntryKind::check(),
    );

    run_str(tran, CREATE).await
}

async fn init_save_register(tran: &mut Transaction<'_>) -> Result<()> {
    const CREATE: &str = const_format::formatcp!(
        "
        CREATE TABLE braid_obj.save_register (
            id bytea PRIMARY KEY,

            CHECK (octet_length(id) = {0})
        );
        ",
        OID_LEN_STR
    );

    run_str(tran, CREATE).await?;
    Ok(())
}

async fn init_save_register_entry(tran: &mut Transaction<'_>) -> Result<()> {
    const CREATE: &str = "
        CREATE TABLE braid_obj.save_register_entry (
            save_register bytea NOT NULL,
            key varchar(255) NOT NULL,
            save bytea NOT NULL,

            FOREIGN KEY (save_register) REFERENCES braid_obj.save_register (id),
            FOREIGN KEY (save) REFERENCES braid_obj.save (id),

            CHECK (
                key NOT LIKE E'%\\n%' AND
                key NOT LIKE E'%\\r%'),

            PRIMARY KEY (save_register, key, save)
        );";

    run_str(tran, CREATE).await
}

async fn init_register_entry(tran: &mut Transaction<'_>) -> Result<()> {
    const CREATE: &str = "
        CREATE TABLE braid_obj.register_entry (
            register bytea NOT NULL,
            key varchar(255) NOT NULL,
            content bytea NOT NULL,
            is_executable boolean NOT NULL,

            FOREIGN KEY (register) REFERENCES braid_obj.register (id),
            FOREIGN KEY (content) REFERENCES braid_obj.content_register (id),

            CHECK (
                key NOT LIKE E'%\\n%' AND
                key NOT LIKE E'%\\r%' AND
                key NOT LIKE '%/%'),

            PRIMARY KEY (key, content)
        );
        ";

    run_str(tran, CREATE).await
}

async fn init_commit(tran: &mut Transaction<'_>) -> Result<()> {
    const CREATE: &str = "
        CREATE TABLE braid_obj.commit (
            id bytea PRIMARY KEY,
            register bytea NOT NULL,
            parent bytea,
            merge_parent bytea,
            rebase_of bytea,
            saves bytea NOT NULL,
            date timestamp with time zone NOT NULL,
            committer varchar(255) NOT NULL,
            summary text NOT NULL,
            body text NOT NULL,

            FOREIGN KEY (id) REFERENCES braid_obj.save_parent (id),
            FOREIGN KEY (register) REFERENCES braid_obj.register (id),
            FOREIGN KEY (parent) REFERENCES braid_obj.commit (id),
            FOREIGN KEY (merge_parent) REFERENCES braid_obj.commit (id),
            FOREIGN KEY (rebase_of) REFERENCES braid_obj.commit (id),
            FOREIGN KEY (saves) REFERENCES braid_obj.save_register (id)
        );";

    run_str(tran, CREATE).await
}

async fn insert_base_data(tran: &mut Transaction<'_>) -> Result<()> {
    let query = sqlx::query("INSERT INTO braid_obj.save_register (id) VALUES ($1);")
        .bind(SaveRegister::EMPTY_ID.as_bytes());
    run(tran, query).await?;

    let query =
        sqlx::query("INSERT INTO braid_obj.content_register (id, is_content) VALUES ($1, false);")
            .bind(Register::EMPTY_ID.as_bytes());
    run(tran, query).await?;

    use super::write_to_tran::Write;
    let _id = CommitData::ROOT.write(tran).await?;

    debug_assert_eq!(_id, Commit::ROOT_ID);

    Ok(())
}

async fn run_str(tran: &mut Transaction<'_>, query: &str) -> Result<()> {
    run(tran, sqlx::query(query)).await
}

async fn run(tran: &mut Transaction<'_>, query: Query<'_>) -> Result<()> {
    query.execute(&mut **tran).await?;
    Ok(())
}

macro_rules! impl_check {
    ($name:ident) => {
        impl $name {
            const fn check() -> &'static str {
                use crate::kind::Kind;

                const ZERO: u8 = b'0';
                const COMMA: u8 = b',';
                const LEN: usize = $name::VARIANTS.len();

                assert!(LEN < 10, "todo: we could handle it if we need to");

                const fn bytes() -> [u8; 2 * LEN - 1] {
                    let mut bytes = [0; 2 * LEN - 1];
                    let mut i = 0;

                    while {
                        bytes[i] = ZERO + $name::VARIANTS[i / 2] as u8;
                        i += 1;
                        i < bytes.len()
                    } {
                        bytes[i] = COMMA;
                        i += 1;
                    }
                    bytes
                }

                const BYTES: &[u8] = &bytes();

                // SAFETY: we know the bytes are valid utf8 and ascii
                unsafe { std::str::from_utf8_unchecked(BYTES) }
            }
        }
    };

    ($($name:ident),+ $(,)?) => {
        $(impl_check!($name);)*
    };
}

impl_check!(RegisterEntryKind, SaveParentKind);
