macro_rules! semi {
    ($lit:literal) => {
        include_str!($lit).split(';')
    };
}

pub(crate) fn init_statements() -> impl Iterator<Item = &'static str> {
    semi!("./init/tables.sql").chain(semi!("./init/views.sql"))
}
