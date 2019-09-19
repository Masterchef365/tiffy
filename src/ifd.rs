use crate::constants::ifd_entry_type_magic::*;
use crate::raw_ifd::{RawIFD, RawIFDEntry};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor, Seek, SeekFrom};

/// Decide whether or not the specified count of this tag type exceeds the 4-byte
/// 'value_or_offset' field within the IFD tag entry.
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

        // Otherwise, assume it fits
        _ => false,
    }
}

/// Convert a TIFF Ascii sequence to string slices
fn iterate_null_terminated_ascii_as_utf8(bytes: &[u8]) -> impl Iterator<Item = &str> {
    bytes
        .split(|x| *x == b'\0')
        .filter(|x| !x.is_empty())
        .map(|x| std::str::from_utf8(x).unwrap()) //TODO: Handle non-Utf8 strings
}

/// IFD Entry Data, essentially a dynamic type representing TIFF's array fields.
#[derive(Debug, Clone, PartialEq)]
pub enum IFDEntryData {
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
    /// The `type` field of the tag was unrecognized when reading
    Unrecognized {
        tag_type: u16,
        count: usize,
        value_or_offset: [u8; 4],
    },
}

impl IFDEntryData {
    /// Read the content from `reader` into this IFDEntryData, dereferencing offset pointers from `entry`
    pub(crate) fn read_fields_from<E: ByteOrder, R: ReadBytesExt + Seek>(
        reader: &mut R,
        entry: &RawIFDEntry,
    ) -> Result<Self, io::Error> {
        if tag_exceeds_ifd_field(entry.tag_type, entry.count) {
            let tag_data_offset = entry.value_or_offset.as_ref().read_u32::<E>()?;
            reader.seek(SeekFrom::Start(tag_data_offset.into()))?;
            Self::from_raw_entry_reader::<E, R>(reader, entry.tag_type, entry.count as usize)
        } else {
            let mut tag_data_cursor = Cursor::new(entry.value_or_offset);
            Self::from_raw_entry_reader::<E, _>(
                &mut tag_data_cursor,
                entry.tag_type,
                entry.count as usize,
            )
        }
    }

    fn from_raw_entry_reader<E: ByteOrder, R: ReadBytesExt>(
        reader: &mut R,
        tag_type: u16,
        count: usize,
    ) -> Result<Self, io::Error> {
        Ok(match tag_type {
            IFD_TYPE_BYTE => {
                let mut buffer = vec![0; count];
                reader.read_exact(&mut buffer)?;
                IFDEntryData::Byte(buffer.into_boxed_slice())
            }
            IFD_TYPE_ASCII => {
                let mut buffer = vec![0; count];
                reader.read_exact(&mut buffer)?;
                IFDEntryData::Ascii(
                    iterate_null_terminated_ascii_as_utf8(&buffer)
                        .map(String::from)
                        .collect(),
                )
            }
            IFD_TYPE_SHORT => {
                let mut buffer = vec![0; count];
                reader.read_u16_into::<E>(&mut buffer)?;
                IFDEntryData::Short(buffer.into_boxed_slice())
            }
            IFD_TYPE_LONG => {
                let mut buffer = vec![0; count];
                reader.read_u32_into::<E>(&mut buffer)?;
                IFDEntryData::Long(buffer.into_boxed_slice())
            }
            IFD_TYPE_RATIONAL => {
                let mut rational_buffer = Vec::with_capacity(count);
                for _ in 0..count {
                    rational_buffer.push((reader.read_u32::<E>()?, reader.read_u32::<E>()?));
                }
                IFDEntryData::Rational(rational_buffer.into_boxed_slice())
            }
            IFD_TYPE_UNDEFINED => {
                let mut buffer = vec![0; count];
                reader.read_exact(&mut buffer)?;
                IFDEntryData::Undefined(buffer.into_boxed_slice())
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
                IFDEntryData::Unrecognized {
                    tag_type,
                    count,
                    value_or_offset,
                }
            }
        })
    }

    /// Dump the data from this entry into `writer`
    pub(crate) fn write_fields_into<E: ByteOrder, W: WriteBytesExt + Seek>(
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

    /// Get the `type` of data in this entry, and the value of the `count` field
    pub(crate) fn get_type_and_count(&self) -> (u16, u32) {
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

/// A single entry (tag, field) in an IFD
#[derive(Debug, Clone, PartialEq)]
pub struct IFDEntry {
    pub tag: u16,
    pub data: IFDEntryData,
}

impl IFDEntry {
    /// Read the data from this raw entry, dereferencing offsets/pointers through `reader`
    pub(crate) fn read_fields_from<E: ByteOrder, R: ReadBytesExt + Seek>(
        reader: &mut R,
        entry: &RawIFDEntry,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            tag: entry.tag,
            data: IFDEntryData::read_fields_from::<E, R>(reader, entry)?,
        })
    }

    /// Convert this entry into a raw one, writing long data to `writer`
    pub(crate) fn write_fields_to<E: ByteOrder, W: WriteBytesExt + Seek>(
        &self,
        writer: &mut W,
    ) -> Result<RawIFDEntry, io::Error> {
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

        Ok(RawIFDEntry {
            tag: self.tag,
            tag_type,
            count,
            value_or_offset,
        })
    }
}

/// A high-level representation of an Image File Directory, including long-format content.
#[derive(Debug, Clone, PartialEq)]
pub struct IFD {
    pub entries: Vec<IFDEntry>,
}

impl IFD {
    /// Read the fields from `reader` into memory, (de)referencing information from `raw_ifd`
    pub(crate) fn read_fields_from<E: ByteOrder, R: ReadBytesExt + Seek>(
        reader: &mut R,
        raw_ifd: &RawIFD,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            entries: raw_ifd
                .entries
                .iter()
                .map(|entry| IFDEntry::read_fields_from::<E, R>(reader, entry))
                .collect::<Result<Vec<IFDEntry>, io::Error>>()?,
        })
    }

    /// Write the fields into `writer`, returning a RawIFD table describing their locations
    pub(crate) fn write_fields_to<E: ByteOrder, W: WriteBytesExt + Seek>(
        &self,
        writer: &mut W,
    ) -> Result<RawIFD, io::Error> {
        Ok(RawIFD {
            entries: self
                .entries
                .iter()
                // Do not write tag types we do not recognize, as it is impossible to do so correctly.
                .filter(|entry| match entry.data {
                    IFDEntryData::Unrecognized{ .. } => false,
                    _ => true,
                })
                .map(|entry| entry.write_fields_to::<E, W>(writer))
                .collect::<Result<Vec<RawIFDEntry>, io::Error>>()?,
        })
    }

    pub fn get_tag(&self, tag: u16) -> Option<&IFDEntryData> {
        self.entries.iter().find(|x| x.tag == tag).map(|x| &x.data)
    }

    pub fn get_tag_mut(&mut self, tag: u16) -> Option<&mut IFDEntryData> {
        self.entries
            .iter_mut()
            .find(|x| x.tag == tag)
            .map(|x| &mut x.data)
    }
}
