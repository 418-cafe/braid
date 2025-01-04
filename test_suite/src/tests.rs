mod hash;
mod init;
mod save;
mod setup;

use braid::Braid;

pub const MAIN: braid::EntryKey = Braid::DEFAULT_MAINLINE;

#[derive(Clone, Copy)]
pub struct Object<'a>(&'a str);

impl braid::Hash for Object<'_> {
    fn hash<H: braid::Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher);
    }
}

macro_rules! db_test {
    ($path:path => $test:ident) => {
        #[tokio::test]
        async fn $test() {
            let (db, pool) = crate::setup::test_database().await;
            $path(pool).await;
            db.drop().await;
        }
    };
}

db_test!(init::test_init => test_init);

db_test!(save::test_save_noop => test_save_noop);
db_test!(save::test_save_serial => test_save_serial);
db_test!(save::test_save_missing_parent => test_save_missing_parent);
db_test!(save::test_save_mismatched_parent => test_save_mismatched_parent);
db_test!(save::test_save_lineage => test_save_lineage);
