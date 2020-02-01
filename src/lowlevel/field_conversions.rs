use crate::errors::FieldExtractionError;
use crate::lowlevel::{IFDField, IFD};
use std::convert::TryInto;

impl IFD {
    pub fn get<'a, T>(&'a self, tag: u16) -> Result<T, FieldExtractionError>
    where
        &'a IFDField: TryInto<T, Error = FieldExtractionError>,
    {
        self.entries
            .get(&tag)
            .ok_or(FieldExtractionError::MissingTag { tag })?
            .try_into()
    }
}

macro_rules! impl_ifdfield_conv {
    { $t:ty, $v:path } => {
        impl<'a> TryInto<&'a [$t]> for &'a IFDField {
            type Error = FieldExtractionError;
            fn try_into(self) -> Result<&'a [$t], Self::Error> {
                match self {
                    $v(val) => Ok(val),
                    _ => Err(FieldExtractionError::WrongDataType),
                }
            }
        }

        impl<'a> TryInto<&'a $t> for &'a IFDField {
            type Error = FieldExtractionError;
            fn try_into(self) -> Result<&'a $t, Self::Error> {
                let array: &[$t] = self.try_into()?;
                array.get(0).ok_or(FieldExtractionError::InsufficientData)
            }
        }

        impl<'a> TryInto<$t> for &'a IFDField {
            type Error = FieldExtractionError;
            fn try_into(self) -> Result<$t, Self::Error> {
                let array: &[$t] = self.try_into()?;
                Ok(array.get(0).ok_or(FieldExtractionError::InsufficientData)?.clone())
            }
        }


    };
}

impl_ifdfield_conv!(u8, IFDField::Byte);
impl_ifdfield_conv!(u16, IFDField::Short);
impl_ifdfield_conv!(u32, IFDField::Long);
impl_ifdfield_conv!(String, IFDField::Ascii);
impl_ifdfield_conv!((u32, u32), IFDField::Rational);
