pub(crate) fn init_statements() -> impl Iterator<Item = &'static str> {
    include_str!("./init/tables.sql").split(';')
}
