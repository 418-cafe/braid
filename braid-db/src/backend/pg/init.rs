use super::PgTransaction;
use crate::{err::Error, register::RegisterEntryKind, save::SaveParentKind, Result};

const ERR_DUPLICATE_SCHEMA: &str = "42P06";
const OID_LEN_STR: &'static str = const_format::concatcp!(braid_hash::Oid::LEN);

pub(super) async fn init(tran: &mut PgTransaction<'_>) -> Result<()> {
    init_schema(tran).await?;
    init_content(tran).await?;
    init_save_parent(tran).await?;
    init_save(tran).await?;
    init_save_register(tran).await?;
    init_save_register_entry(tran).await?;
    init_register_entry(tran).await?;
    init_register(tran).await?;
    init_register_register_entry(tran).await?;
    init_commit(tran).await?;

    Ok(())
}

async fn init_schema(tran: &mut PgTransaction<'_>) -> Result<()> {
    const CREATE: &str = "CREATE SCHEMA obj;";

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

async fn init_content(tran: &mut PgTransaction<'_>) -> Result<()> {
    const CREATE: &str = const_format::formatcp!(
        "
        CREATE TABLE obj.content (
            id bytea PRIMARY KEY,

            CONSTRAINT content_id_len_check CHECK (octet_length(id) = {0})
        );
        ",
        OID_LEN_STR,
    );

    run(tran, CREATE).await
}

async fn init_save_parent(tran: &mut PgTransaction<'_>) -> Result<()> {
    const CREATE: &str = const_format::formatcp!(
        "
        CREATE TABLE obj.save_parent (
            id bytea PRIMARY KEY,
            kind smallint NOT NULL,

            CONSTRAINT save_parent_id_len_check CHECK (octet_length(id) = {0}),
            CONSTRAINT save_parent_kind_check CHECK (kind IN ({1}))
        );
        ",
        OID_LEN_STR,
        SaveParentKind::check(),
    );

    run(tran, CREATE).await
}

async fn init_save(tran: &mut PgTransaction<'_>) -> Result<()> {
    const CREATE: &str = const_format::formatcp!(
        "
            CREATE TABLE obj.save (
                id bytea PRIMARY KEY,
                author varchar(255) NOT NULL,
                date timestamp with time zone NOT NULL,
                kind smallint NOT NULL,
                content bytea NOT NULL,
                parent bytea NOT NULL,

                CONSTRAINT save_id_fkey FOREIGN KEY (id) REFERENCES obj.save_parent (id),
                CONSTRAINT save_save_parent_fkey FOREIGN KEY (parent) REFERENCES obj.save_parent (id),
                CONSTRAINT save_content_fkey FOREIGN KEY (content) REFERENCES obj.content (id),

                CONSTRAINT save_kind_check CHECK (kind IN ({0}))
            );
        ",
        RegisterEntryKind::check(),
    );

    run(tran, CREATE).await
}

async fn init_save_register(tran: &mut PgTransaction<'_>) -> Result<()> {
    const CREATE: &str = const_format::formatcp!(
        "
        CREATE TABLE obj.save_register (
            id bytea PRIMARY KEY,

            CONSTRAINT register_id_len_check CHECK (octet_length(id) = {0})
        );
        ",
        OID_LEN_STR
    );

    run(tran, CREATE).await
}

async fn init_save_register_entry(tran: &mut PgTransaction<'_>) -> Result<()> {
    const CREATE: &str =
        "
        CREATE TABLE obj.save_register_entry (
            save_register bytea NOT NULL,
            key varchar(255) NOT NULL,
            save bytea NOT NULL,

            CONSTRAINT save_register_entry_save_register_fkey FOREIGN KEY (save_register) REFERENCES obj.save_register (id),
            CONSTRAINT save_register_entry_save_fkey FOREIGN KEY (save) REFERENCES obj.save (id),

            CONSTRAINT save_register_entry_key_valid_chars CHECK (
                key NOT LIKE '%\\0%' AND
                key NOT LIKE '%\\n%' AND
                key NOT LIKE '%\\r%'),

            PRIMARY KEY (save_register, key, save)
        );";

    run(tran, CREATE).await
}

async fn init_register_entry(tran: &mut PgTransaction<'_>) -> Result<()> {
    const CREATE: &str = const_format::formatcp!(
        "
        CREATE TABLE obj.register_entry (
            key varchar(255) NOT NULL,
            kind smallint NOT NULL,
            content bytea NOT NULL,

            CONSTRAINT register_entry_content_fkey FOREIGN KEY (content) REFERENCES obj.content (id),
            CONSTRAINT register_entry_kind_check CHECK (kind IN ({0})),

            CONSTRAINT register_entry_key_valid_chars CHECK (
                key NOT LIKE '%\\0%' AND
                key NOT LIKE '%\\n%' AND
                key NOT LIKE '%\\r%' AND
                key NOT LIKE '%/%'),

            PRIMARY KEY (key, kind, content)
        );
    ",
        RegisterEntryKind::check(),
    );

    run(tran, CREATE).await
}

async fn init_register(tran: &mut PgTransaction<'_>) -> Result<()> {
    const CREATE: &str = const_format::formatcp!(
        "
        CREATE TABLE obj.register (
            id bytea PRIMARY KEY,

            CONSTRAINT register_id_len_check CHECK (octet_length(id) = {0})
        );
        ",
        OID_LEN_STR
    );

    run(tran, CREATE).await
}

async fn init_register_register_entry(tran: &mut PgTransaction<'_>) -> Result<()> {
    const CREATE: &str = "
        CREATE TABLE obj.register_register_entry (
            register bytea NOT NULL,
            key varchar(255) NOT NULL,
            kind smallint NOT NULL,
            content bytea NOT NULL,

            CONSTRAINT register_register_entry_register_fkey FOREIGN KEY (register) REFERENCES obj.register (id),
            CONSTRAINT register_register_entry_key_fkey FOREIGN KEY (key, kind, content) REFERENCES obj.register_entry (key, kind, content),

            PRIMARY KEY (register, key, kind, content)
        );";

    run(tran, CREATE).await
}

async fn init_commit(tran: &mut PgTransaction<'_>) -> Result<()> {
    const CREATE: &str =
        "
        CREATE TABLE obj.commit (
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

            CONSTRAINT commit_id_fkey FOREIGN KEY (id) REFERENCES obj.save_parent (id),
            CONSTRAINT commit_register_fkey FOREIGN KEY (register) REFERENCES obj.register (id),
            CONSTRAINT commit_parent_fkey FOREIGN KEY (parent) REFERENCES obj.commit (id),
            CONSTRAINT commit_merge_parent_fkey FOREIGN KEY (merge_parent) REFERENCES obj.commit (id),
            CONSTRAINT commit_rebase_of_fkey FOREIGN KEY (rebase_of) REFERENCES obj.commit (id),
            CONSTRAINT commit_saves_fkey FOREIGN KEY (saves) REFERENCES obj.save_register (id)
        );";

    run(tran, CREATE).await
}

async fn run(tran: &mut PgTransaction<'_>, query: &str) -> Result<()> {
    sqlx::query(query).execute(&mut **tran).await?;
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
