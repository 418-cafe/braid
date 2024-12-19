mod branch;
mod commit;
mod save;

pub use branch::Branch;
pub use commit::{Commit, CommitImpl, CommitWithImpl};
pub use save::Save;

pub(crate) use branch::BranchExists;
pub(crate) use commit::NewCommit;
pub(crate) use save::SaveData;

pub(crate) struct User<'a>(pub(crate) &'a str);
