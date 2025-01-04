use braid::{Braid, CommitWithImpl, InitOptions, Timing};
use sqlx::{types::chrono, PgPool};

pub async fn test_init(pool: PgPool) {
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

    let braid = Braid::init(pool, opts).await.unwrap();

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
}
