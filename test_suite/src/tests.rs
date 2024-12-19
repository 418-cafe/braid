use braid::{Braid, CommitWithImpl, Hash, InitOptions, Key, Timing};
use sqlx::types::chrono::{self, TimeZone};

mod setup;

#[derive(Clone, Copy)]
struct Object;

impl braid::Hash for Object {
    fn hash<H: braid::Hasher>(&self, hasher: &mut H) {
        "hello".hash(hasher);
    }
}

#[tokio::test]
async fn test_init() {
    let mut db = setup::test_database().await;

    let mut tx = db.begin().await;
    let mut braid = Braid::open(tx.get_mut());

    let when = chrono::NaiveDate::from_ymd_opt(2000, 01, 01).expect("invalid date");
    let when = chrono::NaiveDateTime::new(
        when,
        chrono::NaiveTime::from_num_seconds_from_midnight_opt(0, 0).expect("invalid time"),
    );
    let when = chrono::DateTime::from_naive_utc_and_offset(
        when,
        chrono::FixedOffset::east_opt(0).expect("invalid offset"),
    );

    let mut opts = InitOptions::default();
    opts.tz = Some(Timing::When(when));

    braid.init(opts).await.unwrap();

    let CommitWithImpl { commit, commit_impl } = braid.commits().get_root().await.expect("root not found");

    assert_eq!(commit_impl.id(), commit.id());

    assert_eq!(commit.author(), Braid::DEFAULT_USER);
    assert_eq!(commit.subject(), None);
    assert_eq!(commit.body(), None);
    assert_eq!(commit.when(), &when);

    assert_eq!(commit_impl.committer(), Braid::DEFAULT_USER);
    assert_eq!(commit_impl.ancestry(), &braid::Ancestry::Root);
    assert_eq!(commit_impl.committed(), &when);

    tx.rollback().await;
    db.drop().await;
}

#[tokio::test]
async fn test_save() {
    let object = Object;
    let hash = Braid::hash(&object);

    let mut db = setup::test_database().await;

    let mut tx = db.begin().await;
    let mut braid = Braid::open(tx.get_mut());

    braid.init_default().await.unwrap();

    let save = braid
        .save(Key::new("my_object").unwrap(), Braid::DEFAULT_MAINLINE, &object, None)
        .await
        .unwrap();

    println!("{:?}", save);
}
