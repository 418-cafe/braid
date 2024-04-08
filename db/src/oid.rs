use hash::Oid;

use crate::ObjectKind;

trait ValidOid {
    const KIND: ObjectKind;

    fn oid(&self) -> &Oid;
}

macro_rules! impl_validated_oid {
    ($name:ident ($kind:ident)) => {
        pub struct $name(Oid);

        impl $name {
            pub(crate) fn new(oid: Oid) -> Self {
                Self(oid)
            }
        }

        impl ValidOid for $name {
            const KIND: ObjectKind = crate::ObjectKind::$kind;

            fn oid(&self) -> &Oid {
                &self.0
            }
        }
    };
}

impl_validated_oid!(CommitOid(Commit));
impl_validated_oid!(RegisterOid(Register));
impl_validated_oid!(SaveOid(Save));
impl_validated_oid!(SaveRegisterOid(SaveRegister));
