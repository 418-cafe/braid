use braid::{Braid, CommitWithImpl, Hash, InitOptions, Key, Timing};
use sqlx::types::chrono::{self};

mod setup;

#[derive(Clone, Copy)]
struct Object(&'static str);

impl braid::Hash for Object {
    fn hash<H: braid::Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher);
    }
}

const HASH: [&str; 6] = [
    "hello, world!",
    "Hello, world!",
    "foo",
    "bar",
    "foobar",
    "foo bar",
];

#[test]
fn hash_deterministic() {
    assert!{
        HASH
            .map(Object)
            .map(|object| (object, object))
            .map(|(object1, object2)| (Braid::hash(&object1), Braid::hash(&object2)))
            .into_iter()
            .filter(|(left, right)| left != right)
            .next()
            .is_none()
    }
}

#[test]
fn hashes_not_eq() {
    let [current, rest @ ..] = HASH;
    let mut current = Braid::hash(current);

    for next in rest {
        let next = Braid::hash(next);
        assert_ne!(current, next);
        current = next;
    }
}

macro_rules! mk_test {
    (async fn $name:ident($db:ident) $tt:tt) => {
        #[tokio::test]
        async fn $name() {
            let mut $db = setup::test_database().await;
            {
                $tt
            }
            $db.drop().await;
        }
    };
}

mk_test!(async fn test_init(db) {
    let mut tx = db.begin().await;

    let when = chrono::NaiveDate::from_ymd_opt(2000, 01, 01).expect("invalid date");
    let when = chrono::NaiveDateTime::new(
        when,
        chrono::NaiveTime::from_num_seconds_from_midnight_opt(0, 0).expect("invalid time"),
    );
    let when = chrono::DateTime::from_naive_utc_and_offset(
        when,
        chrono::FixedOffset::east_opt(0).expect("invalid offset"),
    )
    .into();

    let mut opts = InitOptions::default();
    opts.tz = Some(Timing::When(when));

    let mut braid = Braid::init(&mut tx, opts).await.unwrap();

    let CommitWithImpl {
        commit,
        commit_impl,
    } = braid.commits().get_root().await.expect("root not found");

    assert_eq!(commit_impl.id(), commit.id());

    assert_eq!(commit.author(), Braid::DEFAULT_USER);
    assert_eq!(commit.subject(), None);
    assert_eq!(commit.body(), None);
    assert_eq!(commit.when(), &when);

    assert_eq!(commit_impl.committer(), Braid::DEFAULT_USER);
    assert_eq!(commit_impl.ancestry(), &braid::Ancestry::Root);
    assert_eq!(commit_impl.committed(), &when);
});

mk_test!(async fn test_save(db) {
    let object = Object("test");

    let mut tx = db.begin().await;
    Braid::init_default(&mut tx).await.unwrap();
    tx.commit().await.unwrap();

    {
        let mut tx = db.begin().await;
        let mut braid = Braid::open(&mut tx);
        let key = Key::new("my_object").unwrap();
        let save = braid
            .save(key, Braid::DEFAULT_MAINLINE, &object, None, None)
            .await
            .expect("first save should be successful");
    
        let next = braid.save(key, Braid::DEFAULT_MAINLINE, &object, None, None).await;
        assert!(matches!(next, Err(braid::Error::MismatchedParent)), "{next:?}");
    }

    {
        let mut tx = db.begin().await;
        let mut braid = Braid::open(&mut tx);
        let key = Key::new("my_object").unwrap();
        let save = braid
            .save(key, Braid::DEFAULT_MAINLINE, &object, None, None)
            .await
            .expect("first save should be successful");
    
        let next = braid.save(key, Braid::DEFAULT_MAINLINE, &object, None, Some(save.id())).await.expect("second save should succeed");
        println!("{next:?}");
    }
});
