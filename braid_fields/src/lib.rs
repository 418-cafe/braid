pub mod commit;

pub(crate) const fn unquote(s: &'static str) -> &'static str {
    match std::str::from_utf8(s.as_bytes().split_at(1).1.split_at(s.len() - 2).0) {
        Ok(s) => s,
        Err(_) => unreachable!(),
    }
}