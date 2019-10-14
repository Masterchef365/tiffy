use crate::errors::FieldExtractionError;
use crate::lowlevel::{IFDField, IFD};
use std::convert::{TryFrom, TryInto};

impl IFD {
    pub fn get<T>(&self, tag: u16) -> Result<T, FieldExtractionError>
    where
        IFDField: TryInto<T, Error = FieldExtractionError>,
    {
        self.entries
            .get(&tag)
            .cloned()
            .ok_or(FieldExtractionError::MissingTag { tag })?
            .try_into()
    }
}

macro_rules! impl_ifdfield_conv {
    { $t:ty, $v:path } => {
        impl TryInto<$t> for IFDField {
            type Error = FieldExtractionError;
            fn try_into(self) -> Result<$t, Self::Error> {
                match self {
                    $v(val) => val
                        .get(0)
                        .copied()
                        .ok_or(FieldExtractionError::InsufficientData),
                    _ => Err(FieldExtractionError::WrongDataType),
                }
            }
        }

        impl TryInto<Box<[$t]>> for IFDField {
            type Error = FieldExtractionError;
            fn try_into(self) -> Result<Box<[$t]>, Self::Error> {
                match self {
                    $v(val) => Ok(val),
                    _ => Err(FieldExtractionError::WrongDataType),
                }
            }
        }

    };
}

impl_ifdfield_conv!(u8, IFDField::Byte);
impl_ifdfield_conv!(u16, IFDField::Short);
impl_ifdfield_conv!(u32, IFDField::Long);
impl_ifdfield_conv!((u32, u32), IFDField::Rational);
