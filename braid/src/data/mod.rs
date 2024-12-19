mod commits;

pub use commits::Commit;

pub(crate) struct BranchExists<'a>(pub(crate) &'a str);

pub(crate) struct User<'a>(pub(crate) &'a str);