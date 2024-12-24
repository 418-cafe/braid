use braid::{Braid, CommitWithImpl, Error, InitOptions, Key, SaveRequest, Timing};
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
    assert! {
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
    (async fn $name:ident($pool:ident) $tt:tt) => {
        #[tokio::test]
        async fn $name() {
            #[allow(dead_code)]
            const MAIN: Key = Braid::DEFAULT_MAINLINE;

            let (db, $pool) = setup::test_database().await;
            {
                $tt
            }

            db.drop().await;
        }
    };
}

mk_test!(async fn test_init(pool) {
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

    let mut braid = Braid::init(pool, opts).await.unwrap();

    let mut tx = braid.begin().await.unwrap();

    let CommitWithImpl {
        commit,
        commit_impl,
    } = tx.commits().get_root().await.expect("root not found");

    assert_eq!(commit_impl.id(), commit.id());

    assert_eq!(commit.author(), Braid::DEFAULT_USER);
    assert_eq!(commit.subject(), None);
    assert_eq!(commit.body(), None);
    assert_eq!(commit.when(), &when);

    assert_eq!(commit_impl.committer(), Braid::DEFAULT_USER);
    assert_eq!(commit_impl.ancestry(), &braid::Ancestry::Root);
    assert_eq!(commit_impl.committed(), &when);
});

mk_test!(async fn test_save_noop(pool) {
    let object = Object("test");

    let mut braid = Braid::init_default(pool).await.unwrap();

    let mut tx = braid.begin().await.unwrap();
    let key = Key::new("my_object").unwrap();

    let request = SaveRequest { key, branch: MAIN, object: Some(&object), tz: None, parent_content: None };

    tx
        .save(request)
        .await
        .expect("first save should be successful");

    assert!(
        tx
            .save(request)
            .await
            .expect("second save should succeed")
            .is_none(),

        "second save should be a noop"
    );

    tx.rollback().await.unwrap();
});

mk_test!(async fn test_save_serial(pool) {
    let object = Object("test");

    let mut braid = Braid::init_default(pool).await.unwrap();

    let mut tx = braid.begin().await.unwrap();
    let key = Key::new("my_object").unwrap();

    let mut request = SaveRequest { key, branch: MAIN, object: Some(&object), tz: None, parent_content: None };

    let parent = tx
        .save(request)
        .await
        .expect("first save should be successful")
        .expect("first save should be Some");

    assert!(parent.content().is_some());

    request.object = Some(&Object("test2"));
    request.parent_content = parent.content();

    let save = tx
        .save(request)
        .await
        .expect("second save should be successful")
        .expect("second save should be Some");

    assert_eq!(save.parent(), Some(parent.id()));

    tx.rollback().await.unwrap();
});

mk_test!(async fn test_save_missing_parent(pool) {
    let object = Object("test");

    let mut braid = Braid::init_default(pool).await.unwrap();

    let mut tx = braid.begin().await.unwrap();
    let key = Key::new("my_object").unwrap();

    let mut request = SaveRequest { key, branch: MAIN, object: Some(&object), tz: None, parent_content: None };

    let parent = tx
        .save(request)
        .await
        .expect("first save should be successful")
        .expect("first save should be Some");

    assert!(parent.content().is_some());

    request.object = Some(&Object("test2"));

    let save = tx.save(request).await;

    assert!(matches!(save, Err(Error::ExpectedParentContentDoesNotMatch)));

    tx.rollback().await.unwrap();
});

mk_test!(async fn test_save_mismatched_parent(pool) {
    let object = Object("test");

    let mut braid = Braid::init_default(pool).await.unwrap();

    let mut tx = braid.begin().await.unwrap();
    let key = Key::new("my_object").unwrap();

    let mut request = SaveRequest { key, branch: MAIN, object: Some(&object), tz: None, parent_content: None };

    let parent = tx
        .save(request)
        .await
        .expect("first save should be successful")
        .expect("first save should be Some");

    assert!(parent.content().is_some());

    // set request.parent_content to the parent save's id - NOT the content, which is invalid
    request.object = Some(&Object("test2"));
    request.parent_content = Some(parent.id());

    let save = tx.save(request).await;

    assert!(matches!(save, Err(Error::ExpectedParentContentDoesNotMatch)));

    tx.rollback().await.unwrap();
});