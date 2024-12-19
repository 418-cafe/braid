mod commits;

pub use commits::{Commit, CommitImpl, CommitWithImpl};
pub(crate) use commits::NewCommit;

pub(crate) struct BranchExists<'a>(pub(crate) &'a str);

pub(crate) struct User<'a>(pub(crate) &'a str);