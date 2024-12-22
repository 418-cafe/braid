use sqlx::PgPool;

use crate::{
    const_unwrap,
    db::{DatabaseTransaction, Transaction},
    hash::{Hash, HasherImpl},
    models::{BranchExists, NewCommit, User},
    Ancestry, Branch, DateTime, Error, FixedOffset, Key, Oid, Result, Save, SaveData,
};

mod commits;

pub struct Braid {
    pool: PgPool,
}

impl Braid {
    pub const DEFAULT_MAINLINE: Key<'static> = const_unwrap!(Ok of Key::new("main"));

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
        BraidTransaction::init(tx, opts).await?;
        Ok(Self { pool })
    }

    pub async fn begin(&mut self) -> Result<BraidTransaction> {
        let tx = self.pool.begin().await?;
        Ok(BraidTransaction::open(tx))
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
}

pub struct BraidTransaction<'t> {
    db: DatabaseTransaction<'t>,
}

impl<'t> BraidTransaction<'t> {
    pub fn open(tx: Transaction<'t>) -> Self {
        let db = DatabaseTransaction::open(tx);
        Self { db }
    }

    pub fn commits(&mut self) -> commits::Commits<'_, 't> {
        commits::Commits::new(self)
    }
}

impl BraidTransaction<'_> {
    /// Initialize the database with custom options.
    pub(crate) async fn init(tx: Transaction<'_>, opts: InitOptions<'_>) -> Result {
        let InitOptions { default, tz } = opts;

        let mut braid = BraidTransaction {
            db: DatabaseTransaction::open(tx),
        };

        braid.db.init().await?;
        braid.db.persist(&User(Braid::DEFAULT_USER)).await?;

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

        braid.db.persist(&root).await?;
        braid.db.persist(&root_impl).await?;

        let name = default.unwrap_or(Braid::DEFAULT_MAINLINE).as_str();

        braid
            .db
            .persist(&Branch {
                name,
                tip: root.id,
                is_default: true,
            })
            .await?;

        braid.commit().await?;

        Ok(())
    }

    /// Write an object to the database, returning its OID. If the object is not reachable by
    /// the time the transaction is committed, it will be eligible for garbage collection.
    pub async fn write<T: Hash>(&mut self, object: &T) -> Result<Oid> {
        let mut hasher = HasherImpl::new();
        object.hash(&mut hasher);
        let oid = hasher.finalize();
        self.db.write_external_object(oid).await?;
        Ok(oid)
    }

    /// Save an object to the database on the branch, returning the persisted save.
    pub async fn save<'a, T>(
        &mut self,
        key: Key<'a>,
        branch: Key<'a>,
        object: &T,
        tz: Option<FixedOffset>,
        expected_parent: Option<Oid>,
    ) -> Result<Save<&'a str>>
    where
        T: Hash,
    {
        let branch = branch.as_str();
        let key = key.as_str();

        if !self.db.exists(&BranchExists(branch)).await? {
            return Err(Error::BranchDoesNotExist(branch.to_string()));
        }

        let content = self.write(object).await?;
        let when = crate::time::now_with_offset(tz);

        let save = SaveData {
            parent: expected_parent,
            branch,
            key,
            is_current: true,
            when,
            content,
        }
        .hash();

        use crate::db::SaveError;
        self.db.persist(&save).await.map_err(|e| match e {
            SaveError::Sql(error) => Error::from(error),
            SaveError::MismatchedParent => Error::MismatchedParent,
        })?;

        Ok(save)
    }

    pub async fn commit(self) -> Result {
        Ok(self.db.into_inner().commit().await?)
    }

    pub async fn rollback(self) -> Result {
        Ok(self.db.into_inner().rollback().await?)
    }
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
    pub default: Option<Key<'a>>,
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
