use commits::Commits;
use saves::Saves;
use sqlx::{PgExecutor, PgPool, Postgres, Transaction};

use crate::{
    const_unwrap,
    db::{self},
    hash::{Hash, HasherImpl},
    models::{NewCommit, User},
    Ancestry, Branch, DateTime, Error, FixedOffset, Key, Oid, Result, Save, SaveData,
    SaveParentContent,
};

mod commits;
mod saves;

type PgTransaction<'a> = Transaction<'a, Postgres>;

pub struct Braid {
    pool: PgPool,
}

impl Braid {
    pub const DEFAULT_MAINLINE: Key<&'static str> = const_unwrap!(Ok of Key::new("main"));

    pub const DEFAULT_USER: &'static str = "";

    pub fn open(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize the database with default values.
    pub async fn init_default(pool: PgPool) -> Result<Self> {
        Self::init(pool, InitOptions::default()).await
    }

    /// Initialize the database with custom options.
    pub async fn init(pool: PgPool, opts: InitOptions<'_>) -> Result<Self> {
        let tx = pool.begin().await?;
        init(tx, opts).await?;
        Ok(Self { pool })
    }

    pub fn into_inner(self) -> PgPool {
        self.pool
    }

    /// Hash an object, returning its OID. This does not write the object to the database.
    pub fn hash<T: Hash + ?Sized>(object: &T) -> Oid {
        let mut hasher = HasherImpl::new();
        object.hash(&mut hasher);
        hasher.finalize()
    }

    /// Begin a save transaction, this gives access to the OID of the object to be saved before the actual
    /// save itself is committed. This allows persistence of the object externally with its OID with ability
    /// to rollback the save if that fails.
    ///
    /// The write of the object to the database will be rolled back if the transaction is not committed.
    /// # Examples
    /// ```rust,ignore
    /// let tx = braid.begin_save("my-object", &object).await?;
    /// external_service.persist(tx.content_hash(), &object);
    /// tx.commit().await?;
    /// ```
    pub async fn begin_save<'a, T: Hash>(
        &self,
        key: Key<&'a str>,
        object: Option<&T>,
    ) -> Result<SaveTransaction<'a, '_>> {
        let mut tx = self.pool.begin().await?;
        let content_hash = match object {
            Some(object) => Some(write(&mut *tx, object).await?),
            None => None,
        };
        Ok(SaveTransaction {
            key,
            content_hash,
            tx,
        })
    }

    /// Write an object to the database, returning its OID. If the object is not reachable by
    /// the time the transaction is committed, it will be eligible for garbage collection.
    pub async fn write<T: Hash>(&self, object: &T) -> Result<Oid> {
        write(&self.pool, object).await
    }

    pub fn commits(&self) -> Commits<'_> {
        Commits::new(self)
    }

    pub fn saves(&self) -> Saves<'_> {
        Saves::new(self)
    }
}

async fn write<T: Hash>(tx: impl PgExecutor<'_>, object: &T) -> Result<Oid> {
    let mut hasher = HasherImpl::new();
    object.hash(&mut hasher);
    let oid = hasher.finalize();
    db::write_external_object(tx, oid).await?;
    Ok(oid)
}

pub struct SaveTransaction<'a, 't> {
    key: Key<&'a str>,
    content_hash: Option<Oid>,
    tx: PgTransaction<'t>,
}

impl<'a> SaveTransaction<'a, '_> {
    pub fn key(&self) -> Key<&str> {
        self.key
    }

    pub fn content_hash(&self) -> Option<Oid> {
        self.content_hash
    }

    pub async fn commit(
        self,
        branch: Key<&str>,
        timing: Option<Timing>,
        parent_content: SaveParentContent,
    ) -> Result<Option<Save<&'a str>>> {
        let Self {
            key,
            content_hash,
            mut tx,
        } = self;

        let when = Timing::into_datetime_or_now(timing);

        use db::state::State;

        // we allow commits to happen concurrently which wouldn't change the actual state, so we can retry until the actual state changes
        loop {
            let current = db::state::current(&mut *tx, branch, key).await?;
            let parent = match current {
                // no-op scenarios - content currently saved matches what's being saved
                State::None if content_hash.is_none() => return Ok(None),
                State::Committed { content } if content_hash == Some(content) => return Ok(None),
                State::Saved { content, .. } | State::SavedCommitted { content, .. }
                    if content_hash == content =>
                {
                    return Ok(None)
                }

                State::None if parent_content.is_none() => None,
                State::Committed { content } if parent_content == Some(content) => None,
                State::Saved { id, content } if parent_content == content => Some(id),
                State::SavedCommitted { id, content, .. } if parent_content == content => Some(id),

                _ => return Err(Error::ExpectedParentContentDoesNotMatch),
            };

            let content = content_hash;
            let data = SaveData {
                parent,
                when,
                content,
            };

            let id = Braid::hash(&data);

            db::save::persist(&mut *tx, id, data).await?;
            if db::state::save_or_update(&mut *tx, branch, key, id, current).await? {
                tx.commit().await?;
                break Ok(Some(Save {
                    id,
                    key,
                    parent,
                    when,
                    content,
                }));
            }
        }
    }
}

async fn init(mut tx: PgTransaction<'_>, opts: InitOptions<'_>) -> Result {
    let InitOptions { default, tz } = opts;

    db::init(&mut tx).await?;
    db::user::persist(&mut *tx, &User(Braid::DEFAULT_USER)).await?;

    let authored = Timing::into_datetime_or_now(tz);

    let root = NewCommit {
        subject: None,
        body: None,
        author: Braid::DEFAULT_USER,
        authored,
        ancestry: Ancestry::Root,
        committer: Braid::DEFAULT_USER,
        committed: authored,
    };

    let (root, root_impl) = root.hash_and_split();

    db::commit::persist(&mut *tx, &root).await?;
    db::commit_impl::persist(&mut *tx, &root_impl).await?;

    let name = default.unwrap_or(Braid::DEFAULT_MAINLINE);

    db::branch::persist(
        &mut *tx,
        &Branch {
            name,
            tip: root.id,
            is_default: true,
        },
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub enum Timing {
    When(DateTime),
    OffsetNow(FixedOffset),
}

impl Timing {
    fn into_datetime(self) -> DateTime {
        match self {
            Self::When(when) => when,
            Self::OffsetNow(offset) => crate::time::now_with_offset(Some(offset)),
        }
    }

    fn into_datetime_or_now(timing: Option<Self>) -> DateTime {
        timing.map_or_else(crate::time::now_utc, Self::into_datetime)
    }
}

pub struct InitOptions<'a> {
    pub default: Option<Key<&'a str>>,
    pub tz: Option<Timing>,
}

impl InitOptions<'_> {
    pub fn new() -> Self {
        Self {
            default: None,
            tz: None,
        }
    }
}

impl Default for InitOptions<'_> {
    fn default() -> Self {
        Self::new()
    }
}
