use crate::lowlevel::{constants::ifd_field_type_magic::*, raw_ifd::RawIFDField};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor, Seek, SeekFrom};

/// IFD Field Data, essentially a dynamic type representing TIFF's array fields.
#[derive(Debug, Clone, PartialEq)]
pub enum IFDField {
    /// Undefined (but not unrecognized) data. Maybe contain binaries.
    Undefined(Box<[u8]>),
    Byte(Box<[u8]>),
    Ascii(Box<[String]>),
    Short(Box<[u16]>),
    Long(Box<[u32]>),
    Rational(Box<[(u32, u32)]>),
    /// The `type` field of the tag was unrecognized when reading. This variant will be ignored
    /// when writing, as there is no way to know how to write it correctly.
    Unrecognized {
        /// Integer representing the type of this tag.
        tag_type: u16,
        /// Integer representing the quantity (not byte count) of this tag.
        count: u32,
        /// Either the tag's value, or a pointer to a location within the file.
        value_or_offset: [u8; 4],
    },
    /* Sbyte(Box<[i8]>),
    SShort(Box<[i16]>),
    SLong(Box<[i32]>),
    SRational(Box<[(i32, i32)]>),
    Double(Box<[f64]>),
    Float(Box<[f32]>), */
}

impl IFDField {
    /// Read the content from `reader` into this IFDField, dereferencing offset pointers from `field`.
    pub fn read_from<E: ByteOrder, R: ReadBytesExt + Seek>(
        reader: &mut R,
        field: &RawIFDField,
    ) -> Result<Self, io::Error> {
        if tag_exceeds_ifd_field(field.tag_type, field.count) {
            let tag_data_offset = field.value_or_offset.as_ref().read_u32::<E>()?;
            reader.seek(SeekFrom::Start(tag_data_offset.into()))?;
            Self::from_raw_field_reader::<E, R>(reader, field.tag_type, field.count)
        } else {
            let mut tag_data_cursor = Cursor::new(field.value_or_offset);
            Self::from_raw_field_reader::<E, _>(&mut tag_data_cursor, field.tag_type, field.count)
        }
    }

    fn from_raw_field_reader<E: ByteOrder, R: ReadBytesExt>(
        reader: &mut R,
        tag_type: u16,
        count: u32,
    ) -> Result<Self, io::Error> {
        Ok(match tag_type {
            IFD_TYPE_BYTE => {
                let mut buffer = vec![0; count as usize];
                reader.read_exact(&mut buffer)?;
                IFDField::Byte(buffer.into_boxed_slice())
            }
            IFD_TYPE_ASCII => {
                let mut buffer = vec![0; count as usize];
                reader.read_exact(&mut buffer)?;
                IFDField::Ascii(
                    iterate_null_terminated_ascii_as_utf8(&buffer)
                        .map(String::from)
                        .collect(),
                )
            }
            IFD_TYPE_SHORT => {
                let mut buffer = vec![0; count as usize];
                reader.read_u16_into::<E>(&mut buffer)?;
                IFDField::Short(buffer.into_boxed_slice())
            }
            IFD_TYPE_LONG => {
                let mut buffer = vec![0; count as usize];
                reader.read_u32_into::<E>(&mut buffer)?;
                IFDField::Long(buffer.into_boxed_slice())
            }
            IFD_TYPE_RATIONAL => {
                let mut rational_buffer = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    rational_buffer.push((reader.read_u32::<E>()?, reader.read_u32::<E>()?));
                }
                IFDField::Rational(rational_buffer.into_boxed_slice())
            }
            IFD_TYPE_UNDEFINED => {
                let mut buffer = vec![0; count as usize];
                reader.read_exact(&mut buffer)?;
                IFDField::Undefined(buffer.into_boxed_slice())
            }
            IFD_TYPE_SBYTE => unimplemented!("SByte IFD values"),
            IFD_TYPE_SSHORT => unimplemented!("SShort IFD values"),
            IFD_TYPE_SLONG => unimplemented!("SLong IFD values"),
            IFD_TYPE_SRATIONAL => unimplemented!("SRational IFD values"),
            IFD_TYPE_FLOAT => unimplemented!("Float IFD values"),
            IFD_TYPE_DOUBLE => unimplemented!("Double IFD values"),
            _ => {
                let mut value_or_offset = [0u8; 4];
                reader.read_exact(&mut value_or_offset)?;
                IFDField::Unrecognized {
                    tag_type,
                    count,
                    value_or_offset,
                }
            }
        })
    }

    /// Write this IFDField into `writer`
    pub fn write_to<E: ByteOrder, W: WriteBytesExt + Seek>(
        &self,
        writer: &mut W,
        tag: u16,
    ) -> Result<RawIFDField, io::Error> {
        let mut value_or_offset = [0u8; 4];
        let mut cursor = Cursor::new(&mut value_or_offset[..]);

        let tag_type = self.type_number();
        let count = self.count() as u32;
        if tag_exceeds_ifd_field(tag_type, count) {
            let data_offset = writer.seek(SeekFrom::Current(0))? as u32;
            cursor.write_u32::<E>(data_offset)?;
            self.write_field_into::<E, _>(writer)?;
        } else {
            self.write_field_into::<E, _>(&mut cursor)?;
        }

        Ok(RawIFDField {
            tag,
            tag_type,
            count,
            value_or_offset,
        })
    }

    /// Dump the data from this field into `writer`.
    pub fn write_field_into<E: ByteOrder, W: WriteBytesExt + Seek>(
        &self,
        writer: &mut W,
    ) -> Result<(), io::Error> {
        match self {
            Self::Undefined(bytes) => writer.write_all(&bytes),
            Self::Byte(bytes) => writer.write_all(&bytes),
            Self::Ascii(strings) => {
                for string in strings.iter() {
                    writer.write_all(string.as_bytes())?;
                    writer.write_all(&[b'\0'])?;
                }
                Ok(())
            }
            Self::Short(shorts) => shorts
                .iter()
                .try_for_each(|short| writer.write_u16::<E>(*short)),
            Self::Long(longs) => longs
                .iter()
                .try_for_each(|long| writer.write_u32::<E>(*long)),
            Self::Rational(rationals) => rationals.iter().try_for_each(|(a, b)| {
                writer
                    .write_u32::<E>(*a)
                    .and_then(|()| writer.write_u32::<E>(*b))
            }),
            Self::Unrecognized {
                value_or_offset, ..
            } => writer.write_all(value_or_offset),
        }
    }

    /// Get the `type` of data in this field, and the value of the `count` field.
    pub fn count(&self) -> usize {
        match self {
            Self::Undefined(data) => data.len(),
            Self::Byte(data) => data.len(),
            Self::Ascii(strings) => {
                let mut length: usize = 0;
                for string in strings.iter() {
                    length += string.as_bytes().len();
                    length += 1; // For null character
                }
                length
            }
            Self::Short(data) => data.len(),
            Self::Long(data) => data.len(),
            Self::Rational(data) => data.len(),
            Self::Unrecognized { count, .. } => *count as usize,
        }
    }

    pub fn type_number(&self) -> u16 {
        match self {
            Self::Undefined(_) => IFD_TYPE_UNDEFINED,
            Self::Byte(_) => IFD_TYPE_BYTE,
            Self::Ascii(_) => IFD_TYPE_ASCII,
            Self::Short(_) => IFD_TYPE_SHORT,
            Self::Long(_) => IFD_TYPE_LONG,
            Self::Rational(_) => IFD_TYPE_RATIONAL,
            Self::Unrecognized { tag_type, .. } => *tag_type,
        }
    }
}

/// Decide whether or not the specified count of this tag type exceeds the 4-byte
/// 'value_or_offset' field within the IFD tag field.
pub fn tag_exceeds_ifd_field(tag_type: u16, count: u32) -> bool {
    match tag_type {
        // Single byte per unit fields
        IFD_TYPE_BYTE if count > 4 => true,
        IFD_TYPE_ASCII if count > 4 => true,
        IFD_TYPE_SBYTE if count > 4 => true,
        IFD_TYPE_UNDEFINED if count > 4 => true,

        // Two byte per unit fields
        IFD_TYPE_SHORT if count > 2 => true,
        IFD_TYPE_SSHORT if count > 2 => true,

        // Four byte per unit fields
        IFD_TYPE_LONG if count > 1 => true,
        IFD_TYPE_SLONG if count > 1 => true,
        IFD_TYPE_FLOAT if count > 1 => true,

        // 5+ byte per unit fields
        IFD_TYPE_RATIONAL => true,
        IFD_TYPE_SRATIONAL => true,
        IFD_TYPE_DOUBLE => true,

        // Otherwise, assume it fits (As it is unrecognized and custom-defined)
        _ => false,
    }
}

// TODO: Do not ignore non-utf8 strings, or at least warn about these
/// Convert a TIFF Ascii sequence to string slices.
fn iterate_null_terminated_ascii_as_utf8(bytes: &[u8]) -> impl Iterator<Item = &str> {
    bytes
        .split(|x| *x == b'\0')
        .filter(|x| !x.is_empty())
        .filter_map(|x| std::str::from_utf8(x).ok())
}
