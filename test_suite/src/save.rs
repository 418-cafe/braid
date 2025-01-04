use braid::{Braid, Error, FullKey};
use sqlx::PgPool;

use crate::{Object, MAIN};

pub async fn test_save_noop(pool: PgPool) {
    let braid = Braid::init_default(pool).await.unwrap();
    let key = FullKey::new("my_object").unwrap();

    let object = Object("test");

    for expect in [true, false] {
        assert_eq!(
            braid
                .begin_save(key, Some(&object))
                .await
                .expect("save transaction create should succeed")
                .commit(MAIN, None, None)
                .await
                .expect("save should succeed")
                .is_some(),
            expect
        );
    }
}

pub async fn test_save_serial(pool: PgPool) {
    let braid = Braid::init_default(pool).await.unwrap();
    let key = FullKey::new("my_object").unwrap();

    let parent = braid
        .begin_save(key, Some(&Object("1")))
        .await
        .expect("first save transaction create should succeed")
        .commit(MAIN, None, None)
        .await
        .expect("first save should succeed")
        .expect("first save should be some");

    assert!(parent.content().is_some());

    let save = braid
        .begin_save(key, Some(&Object("2")))
        .await
        .expect("second save transaction create should succeed")
        .commit(MAIN, None, parent.content())
        .await
        .expect("second save should succeed")
        .expect("second save should be some");

    assert_eq!(save.parent(), Some(parent.id()));
}

pub async fn test_save_missing_parent(pool: PgPool) {
    let braid = Braid::init_default(pool).await.unwrap();
    let key = FullKey::new("my_object").unwrap();

    let parent = braid
        .begin_save(key, Some(&Object("1")))
        .await
        .expect("first save transaction create should succeed")
        .commit(MAIN, None, None)
        .await
        .expect("first save should succeed")
        .expect("first save should be some");

    assert!(parent.content().is_some());

    let err = braid
        .begin_save(key, Some(&Object("2")))
        .await
        .expect("second save transaction create should succeed")
        .commit(MAIN, None, None)
        .await;

    assert!(matches!(err, Err(Error::ExpectedParentContentDoesNotMatch)));
}

pub async fn test_save_mismatched_parent(pool: PgPool) {
    let braid = Braid::init_default(pool).await.unwrap();
    let key = FullKey::new("my_object").unwrap();

    let parent = braid
        .begin_save(key, Some(&Object("1")))
        .await
        .expect("first save transaction create should succeed")
        .commit(MAIN, None, None)
        .await
        .expect("first save should succeed")
        .expect("first save should be some");

    assert!(parent.content().is_some());

    // set request.parent_content to the parent save's id - NOT the content, which is invalid
    let err = braid
        .begin_save(key, Some(&Object("2")))
        .await
        .expect("second save transaction create should succeed")
        .commit(MAIN, None, Some(parent.id()))
        .await;

    assert!(matches!(err, Err(Error::ExpectedParentContentDoesNotMatch)));
}

pub async fn test_save_lineage(pool: PgPool) {
    let braid = Braid::init_default(pool).await.unwrap();
    let key = FullKey::new("my_object").unwrap();

    let mut parent_content = None;
    let mut stack = Vec::new();

    for i in 0..10 {
        let save = braid
            .begin_save(key, Some(&Object(i.to_string().as_str())))
            .await
            .expect("save transaction create should succeed")
            .commit(MAIN, None, parent_content)
            .await
            .expect("save should succeed")
            .expect("save should be some");

        stack.push(save.id());

        debug_assert!(save.content().is_some());

        parent_content = save.content();
    }

    let lineage: Vec<_> = braid
        .saves()
        .get_lineage(MAIN, [key])
        .await
        .expect("lineage should succeed")
        .collect();

    let mut i = 0;
    while let Some(id) = stack.pop() {
        assert_eq!(lineage[i].id(), id);
        i += 1;
    }
}
