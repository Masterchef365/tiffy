use crate::metadata::{
    constants::ifd_field_type_magic::*,
    raw_ifd::{RawIFD, RawIFDField},
};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor, Seek, SeekFrom};

/// Decide whether or not the specified count of this tag type exceeds the 4-byte
/// 'value_or_offset' field within the IFD tag field.
fn tag_exceeds_ifd_field(tag_type: u16, count: u32) -> bool {
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

        // Otherwise, assume it fits
        _ => false,
    }
}

/// Convert a TIFF Ascii sequence to string slices.
fn iterate_null_terminated_ascii_as_utf8(bytes: &[u8]) -> impl Iterator<Item = &str> {
    bytes
        .split(|x| *x == b'\0')
        .filter(|x| !x.is_empty())
        .filter_map(|x| std::str::from_utf8(x).ok())
}

/// IFD Field Data, essentially a dynamic type representing TIFF's array fields.
#[derive(Debug, Clone, PartialEq)]
pub enum IFDFieldData {
    Undefined(Box<[u8]>),
    Byte(Box<[u8]>),
    Ascii(Box<[String]>),
    Short(Box<[u16]>),
    Long(Box<[u32]>),
    Rational(Box<[(u32, u32)]>),
    /* Sbyte(Box<[i8]>),
    Sshort(Box<[i16]>),
    Slong(Box<[i32]>),
    SRational(Box<[(i32, i32)]>),
    Double(Box<[f64]>),
    Float(Box<[f32]>), */
    /// The `type` field of the tag was unrecognized when reading.
    Unrecognized {
        tag_type: u16,
        count: usize,
        value_or_offset: [u8; 4],
    },
}

impl IFDFieldData {
    /// Read the content from `reader` into this IFDFieldData, dereferencing offset pointers from `field`.
    pub fn read_from<E: ByteOrder, R: ReadBytesExt + Seek>(
        reader: &mut R,
        field: &RawIFDField,
    ) -> Result<Self, io::Error> {
        if tag_exceeds_ifd_field(field.tag_type, field.count) {
            let tag_data_offset = field.value_or_offset.as_ref().read_u32::<E>()?;
            reader.seek(SeekFrom::Start(tag_data_offset.into()))?;
            Self::from_raw_field_reader::<E, R>(reader, field.tag_type, field.count as usize)
        } else {
            let mut tag_data_cursor = Cursor::new(field.value_or_offset);
            Self::from_raw_field_reader::<E, _>(
                &mut tag_data_cursor,
                field.tag_type,
                field.count as usize,
            )
        }
    }

    fn from_raw_field_reader<E: ByteOrder, R: ReadBytesExt>(
        reader: &mut R,
        tag_type: u16,
        count: usize,
    ) -> Result<Self, io::Error> {
        Ok(match tag_type {
            IFD_TYPE_BYTE => {
                let mut buffer = vec![0; count];
                reader.read_exact(&mut buffer)?;
                IFDFieldData::Byte(buffer.into_boxed_slice())
            }
            IFD_TYPE_ASCII => {
                let mut buffer = vec![0; count];
                reader.read_exact(&mut buffer)?;
                IFDFieldData::Ascii(
                    iterate_null_terminated_ascii_as_utf8(&buffer)
                        .map(String::from)
                        .collect(),
                )
            }
            IFD_TYPE_SHORT => {
                let mut buffer = vec![0; count];
                reader.read_u16_into::<E>(&mut buffer)?;
                IFDFieldData::Short(buffer.into_boxed_slice())
            }
            IFD_TYPE_LONG => {
                let mut buffer = vec![0; count];
                reader.read_u32_into::<E>(&mut buffer)?;
                IFDFieldData::Long(buffer.into_boxed_slice())
            }
            IFD_TYPE_RATIONAL => {
                let mut rational_buffer = Vec::with_capacity(count);
                for _ in 0..count {
                    rational_buffer.push((reader.read_u32::<E>()?, reader.read_u32::<E>()?));
                }
                IFDFieldData::Rational(rational_buffer.into_boxed_slice())
            }
            IFD_TYPE_UNDEFINED => {
                let mut buffer = vec![0; count];
                reader.read_exact(&mut buffer)?;
                IFDFieldData::Undefined(buffer.into_boxed_slice())
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
                IFDFieldData::Unrecognized {
                    tag_type,
                    count,
                    value_or_offset,
                }
            }
        })
    }

    /// Dump the data from this field into `writer`.
    pub fn write_fields_into<E: ByteOrder, W: WriteBytesExt + Seek>(
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
            _ => unreachable!("Unrecognized tag types should be filtered before writing"),
        }
    }

    /// Get the `type` of data in this field, and the value of the `count` field.
    pub fn get_type_and_count(&self) -> (u16, u32) {
        match self {
            Self::Undefined(data) => (IFD_TYPE_UNDEFINED, data.len() as u32),
            Self::Byte(data) => (IFD_TYPE_BYTE, data.len() as u32),
            Self::Ascii(strings) => {
                let mut length: u32 = 0;
                for string in strings.iter() {
                    length += string.as_bytes().len() as u32;
                    length += 1; // For null character
                }
                (IFD_TYPE_ASCII, length)
            }
            Self::Short(data) => (IFD_TYPE_SHORT, data.len() as u32),
            Self::Long(data) => (IFD_TYPE_LONG, data.len() as u32),
            Self::Rational(data) => (IFD_TYPE_RATIONAL, data.len() as u32),
            _ => unreachable!("Unrecognized tag types should be filtered before writing"),
        }
    }
}

/// An IFD field.
#[derive(Debug, Clone, PartialEq)]
pub struct IFDField {
    pub tag: u16,
    pub data: IFDFieldData,
}

impl IFDField {
    /// Read the data from this raw field, dereferencing offsets/pointers through `reader`.
    pub fn read_from<E: ByteOrder, R: ReadBytesExt + Seek>(
        reader: &mut R,
        field: &RawIFDField,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            tag: field.tag,
            data: IFDFieldData::read_from::<E, R>(reader, field)?,
        })
    }

    /// Convert this field into a raw one, writing long data to `writer`.
    pub fn write_to<E: ByteOrder, W: WriteBytesExt + Seek>(
        &self,
        writer: &mut W,
    ) -> Result<RawIFDField, io::Error> {
        let mut value_or_offset = [0u8; 4];
        let mut cursor = Cursor::new(&mut value_or_offset[..]);

        let (tag_type, count) = self.data.get_type_and_count();
        if tag_exceeds_ifd_field(tag_type, count) {
            let data_offset = writer.seek(SeekFrom::Current(0))? as u32;
            cursor.write_u32::<E>(data_offset)?;
            self.data.write_fields_into::<E, _>(writer)?;
        } else {
            self.data.write_fields_into::<E, _>(&mut cursor)?;
        }

        Ok(RawIFDField {
            tag: self.tag,
            tag_type,
            count,
            value_or_offset,
        })
    }
}

/// A high-level representation of an Image File Directory.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IFD {
    pub entries: Vec<IFDField>,
}

impl IFD {
    /// Create an empty IFD.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Read the fields from `reader` into memory, (de)referencing information from `raw_ifd`.
    pub fn read_from<E: ByteOrder, R: ReadBytesExt + Seek>(
        reader: &mut R,
        raw_ifd: &RawIFD,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            entries: raw_ifd
                .entries
                .iter()
                .map(|field| IFDField::read_from::<E, R>(reader, field))
                .collect::<Result<Vec<IFDField>, io::Error>>()?,
        })
    }

    /// Write the fields into `writer`, returning a RawIFD describing their locations or data.
    pub fn write_to<E: ByteOrder, W: WriteBytesExt + Seek>(
        &self,
        writer: &mut W,
    ) -> Result<RawIFD, io::Error> {
        Ok(RawIFD {
            entries: self
                .entries
                .iter()
                // Do not write tag types we do not recognize, as it is impossible to do so correctly.
                .filter(|field| match field.data {
                    IFDFieldData::Unrecognized { .. } => false,
                    _ => true,
                })
                .map(|field| field.write_to::<E, W>(writer))
                .collect::<Result<Vec<RawIFDField>, io::Error>>()?,
        })
    }

    pub fn add_tag(&mut self, tag: u16, data: IFDFieldData) {
        self.entries.push(IFDField { tag, data });
    }

    pub fn get_tag(&self, tag: u16) -> Option<&IFDFieldData> {
        self.entries.iter().find(|x| x.tag == tag).map(|x| &x.data)
    }

    pub fn get_tag_mut(&mut self, tag: u16) -> Option<&mut IFDFieldData> {
        self.entries
            .iter_mut()
            .find(|x| x.tag == tag)
            .map(|x| &mut x.data)
    }
}
