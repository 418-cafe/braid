use sqlx::{postgres::PgArguments, query::QueryScalar, Postgres};

use crate::BranchExists;

type QueryExists<'q> = QueryScalar<'q, Postgres, bool, PgArguments>;

pub(crate) trait Exists<'a> {
    const EXISTS: &'static str;

    fn bind(&'a self, query: QueryExists<'a>) -> QueryExists<'a>;
}

impl<'a> Exists<'a> for BranchExists<'a> {
    const EXISTS: &'static str = "SELECT EXISTS(SELECT 1 FROM \"branch\" WHERE \"name\" = $1)";

    fn bind(&'a self, query: QueryExists<'a>) -> QueryExists<'a> {
        query.bind(self.0)
    }
}