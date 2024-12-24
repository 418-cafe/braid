mod branch;
mod commit;
mod request;
mod save;

pub use branch::Branch;
pub use commit::{Commit, CommitImpl, CommitWithImpl};
pub use request::{SaveParentContent, SaveRequest};
pub use save::Save;

pub(crate) use branch::BranchExists;
pub(crate) use commit::NewCommit;
pub(crate) use save::{SaveData, SaveLineageCriteria};

pub(crate) struct User<'a>(pub(crate) &'a str);
