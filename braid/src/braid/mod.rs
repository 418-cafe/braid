use sqlx::types::chrono::FixedOffset;

use crate::{
    const_unwrap,
    db::{Database, Transaction},
    hash::{Hash, HasherImpl},
    models::{BranchExists, NewCommit, User},
    Ancestry, Branch, DateTime, Error, Key, Oid, Result, Save, SaveData,
};

mod commits;

pub struct Braid<'a, 't> {
    db: Database<'a, 't>,
}

impl<'a, 't> Braid<'a, 't> {
    pub fn open(tx: &'a mut Transaction<'t>) -> Self {
        let db = Database::open(tx);
        Self { db }
    }

    /// Initialize the database with default values.
    pub async fn init_default(tx: &'a mut Transaction<'t>) -> Result<Self> {
        Self::init(tx, InitOptions::default()).await
    }

    /// Initialize the database with custom options.
    pub async fn init(tx: &'a mut Transaction<'t>, opts: InitOptions<'_>) -> Result<Self> {
        let InitOptions { default, tz } = opts;

        let mut braid = Self {
            db: Database::open(tx),
        };

        braid.db.init().await?;
        braid.db.persist(&User(Self::DEFAULT_USER)).await?;

        let authored = Timing::into_datetime_or_now(tz);

        let root = NewCommit {
            subject: None,
            body: None,
            author: Self::DEFAULT_USER,
            authored,
            ancestry: Ancestry::Root,
            committer: Self::DEFAULT_USER,
            committed: authored,
        };

        let (root, root_impl) = root.hash_and_split();

        braid.db.persist(&root).await?;
        braid.db.persist(&root_impl).await?;

        let name = default.unwrap_or(Self::DEFAULT_MAINLINE).as_str();

        braid
            .db
            .persist(&Branch {
                name,
                tip: root.id,
                is_default: true,
            })
            .await?;

        Ok(braid)
    }

    pub fn commits(&'a mut self) -> commits::Commits<'a, 't> {
        commits::Commits::new(self)
    }
}

impl Braid<'_, '_> {
    pub const DEFAULT_MAINLINE: Key<'static> = const_unwrap!(Key::new("main"));

    pub const DEFAULT_USER: &'static str = "";

    /// Hash an object, returning its OID. This does not write the object to the database.
    pub fn hash<T: Hash + ?Sized>(object: &T) -> Oid {
        let mut hasher = HasherImpl::new();
        object.hash(&mut hasher);
        hasher.finalize()
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
