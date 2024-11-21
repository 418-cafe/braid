use std::{borrow::Borrow, collections::HashSet, ptr::NonNull};

use crate::{ancestry::Ancestry, oid::hashable_by_oid, ObjectId, Oid};

struct Parent<D>(NonNull<CommitGraphNode<D>>);

impl<D> Parent<D> {
    unsafe fn as_mut(&self) -> &mut CommitGraphNode<D> {
        &mut *self.0.as_ptr()
    }
}

pub(crate) struct CommitGraphNode<D> {
    oid: Oid,
    ancestry: Ancestry<Parent<D>>,
    data: D,
}

hashable_by_oid!(CommitGraphNode<D>);

pub(crate) trait CommitData : ObjectId {
    fn parent(&self) -> &Ancestry<Oid>;
}

/// A fully loaded commit graph. This graph won't reallocate,
/// so the set contains raw pointers to parents in the graph.
pub(crate) struct CommitGraph<D> {
    set: HashSet<CommitGraphNode<D>>,
}

impl<D> CommitGraph<D> {
    /// Create a new commit graph from a list of commits. Commits MUST be ordered by generation ascending.
    pub(crate) fn with_commits_mapped<C: CommitData>(commits: Vec<C>, f: impl Fn(C) -> D) -> Self {
        let mut set = HashSet::with_capacity(commits.len());

        for commit in commits {
            let oid = commit.oid();
            let ancestry = commit.parent().as_ref().map(|parent| {
                let parent: &CommitGraphNode<D> = set.get(parent).expect("Could not find parent.");
                Parent(NonNull::from(parent))
            });
            let data = f(commit);

            set.insert(CommitGraphNode {
                oid,
                ancestry,
                data,
            });
        }

        CommitGraph { set }
    }

    pub(crate) fn get(&self, oid: impl Borrow<Oid>) -> Option<&CommitGraphNode<D>> {
        self.set.get(oid.borrow())
    }

    pub(crate) fn walk_mut(&mut self, reachable_from: impl Iterator<Item = Oid>) -> CommitGraphIterMut<D> {
        // SAFETY: As this is a DAG, we will never have a cycle in the graph, and no node will be mutably
        // borrowed more than once at a time. This allows us to traverse the graph without needing to
        // perform a lookup on the set for each parent.
        let stack = reachable_from
            .filter_map(|oid| self.set.get(&oid))
            .map(|node| node as *const CommitGraphNode<D>)
            .map(|node| unsafe { &mut *(node as *mut CommitGraphNode<D>) })
            .collect();

        CommitGraphIterMut { stack }
    }
}

pub(crate) struct CommitGraphIterMut<'g, D> {
    stack: Vec<&'g mut CommitGraphNode<D>>,
}

impl<D> CommitGraphIterMut<'_, D> {
    pub(crate) fn next(&mut self) -> Option<&mut CommitGraphNode<D>> {
        match self.stack.pop() {
            None => None,

            // SAFETY: The stack only contains mutable references to nodes in the graph.
            // The borrow technically comes off the descendants of the nodes, but it's
            // really a borrow of the graph itself. The user won't have mutable access to
            // the parents through the commit. Only when it is popped off the stack here.
            // And our nodes in the stack our valid for the lifetime of the graph.
            Some(next) => unsafe {
                let next = next as *mut CommitGraphNode<D>;

                if let Ancestry::Parent { ref parent, ref merge_parent } = (*next).ancestry {
                    self.stack.push(parent.as_mut());
                    if let Some(merge_parent) = merge_parent {
                        self.stack.push(merge_parent.as_mut());
                    }
                }

                Some(&mut *next)
            },
        }
    }   
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Commit {
        oid: Oid,
        parent: Ancestry<Oid>,
        data: bool,
    }

    impl ObjectId for Commit {
        fn oid(&self) -> Oid {
            self.oid
        }
    }

    impl CommitData for Commit {
        fn parent(&self) -> &Ancestry<Oid> {
            &self.parent
        }
    }

    #[test]
    fn test1() {
        let root_oid = Oid::new([0; Oid::LEN]);
        let root = Commit {
            oid: root_oid,
            parent: Ancestry::Root,
            data: false,
        };

        let left_oid = Oid::new([1; Oid::LEN]);
        let left = Commit {
            oid: left_oid,
            parent: Ancestry::Parent {
                parent: root.oid,
                merge_parent: None,
            },
            data: false,
        };

        let right_oid = Oid::new([2; Oid::LEN]);
        let right = Commit {
            oid: right_oid,
            parent: Ancestry::Parent {
                parent: root.oid,
                merge_parent: None,
            },
            data: false,
        };

        let mut graph = CommitGraph::with_commits_mapped(vec![root, left, right], |commit| commit.data);

        let mut iter = graph.walk_mut(std::iter::once(right_oid));

        while let Some(commit) = iter.next() {
            commit.data = true;
        }

        {
            let get_data = |oid| graph.get(oid).unwrap().data;
    
            assert_eq!(get_data(left_oid), false);
            assert_eq!(get_data(right_oid), true);
            assert_eq!(get_data(root_oid), true);
        }

        let mut iter = graph.walk_mut(std::iter::once(left_oid));

        while let Some(commit) = iter.next() {
            commit.data = true;
        }

        {
            let get_data = |oid| graph.get(oid).unwrap().data;
    
            assert_eq!(get_data(left_oid), false);
            assert_eq!(get_data(right_oid), true);
            assert_eq!(get_data(root_oid), true);
        }
    }
}