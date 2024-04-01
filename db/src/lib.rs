mod fs;
mod register;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ObjectKind {
    Register = 0,
    Content = 1,
    ExecutableContent = 2,
}

impl ObjectKind {
    pub(crate) fn from_u8(value: u8) -> Option<Self> {
        const REGISTER: u8 = ObjectKind::Register as u8;
        const CONTENT: u8 = ObjectKind::Content as u8;
        const EXECUTABLE_CONTENT: u8 = ObjectKind::ExecutableContent as u8;

        match value {
            REGISTER => Some(ObjectKind::Register),
            CONTENT => Some(ObjectKind::Content),
            EXECUTABLE_CONTENT => Some(ObjectKind::ExecutableContent),
            _ => None,
        }
    }
}

pub(crate) trait Object {
    const KIND: ObjectKind;
}
