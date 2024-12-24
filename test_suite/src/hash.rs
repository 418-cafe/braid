use braid::Braid;

use crate::Object;

const HASH: [&str; 6] = [
    "hello, world!",
    "Hello, world!",
    "foo",
    "bar",
    "foobar",
    "foo bar",
];

#[test]
fn hash_deterministic() {
    assert! {
        HASH
            .map(Object)
            .map(|object| (object, object))
            .map(|(object1, object2)| (Braid::hash(&object1), Braid::hash(&object2)))
            .into_iter()
            .filter(|(left, right)| left != right)
            .next()
            .is_none()
    }
}

#[test]
fn hashes_not_eq() {
    let [current, rest @ ..] = HASH;
    let mut current = Braid::hash(current);

    for next in rest {
        let next = Braid::hash(next);
        assert_ne!(current, next);
        current = next;
    }
}
