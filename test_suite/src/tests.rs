mod hash;
mod save;
mod setup;

macro_rules! mk_test {
    (async fn $name:ident($pool:ident) $tt:tt) => {
        #[tokio::test]
        async fn $name() {
            #[allow(dead_code)]
            const MAIN: Key = Braid::DEFAULT_MAINLINE;

            let (db, $pool) = crate::setup::test_database().await;
            {
                $tt
            }

            db.drop().await;
        }
    };
}
pub(crate) use mk_test;

#[derive(Clone, Copy)]
pub(crate) struct Object<'a>(&'a str);

impl braid::Hash for Object<'_> {
    fn hash<H: braid::Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher);
    }
}
