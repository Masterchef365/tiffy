use crate::metadata::IFDField;
use failure::Fail;
use std::convert::TryInto;

/// An error encountered during conversion from an IFDField to another type
#[derive(Fail, Debug)]
pub enum FieldConvError {
    #[fail(display = "Tag has wrong data type")]
    WrongDataType,
    #[fail(display = "Tag contains insufficient data")]
    InsufficientData,
}

macro_rules! impl_ifdfield_conv {
    { $t:ty, $v:path } => {
        impl From<$t> for IFDField {
            fn from(val: $t) -> Self {
                $v(Box::new([val]))
            }
        }

        impl TryInto<$t> for IFDField {
            type Error = FieldConvError;
            fn try_into(self) -> Result<$t, Self::Error> {
                match self {
                    $v(val) => val
                        .get(0)
                        .copied()
                        .ok_or(FieldConvError::InsufficientData),
                    _ => Err(FieldConvError::WrongDataType),
                }
            }
        }
    };
}

impl_ifdfield_conv!(u8, IFDField::Byte);
impl_ifdfield_conv!(u16, IFDField::Short);
impl_ifdfield_conv!(u32, IFDField::Long);
impl_ifdfield_conv!((u32, u32), IFDField::Rational);

#[test]
fn test_roundtrip_field_conversion() {
    let original = (8u32, 9u32);
    let field: IFDField = original.into();
    let converted: (u32, u32) = field.try_into().unwrap();
    assert_eq!(converted, original);
}
