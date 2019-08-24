use crate::raw_ifd::{RawIFD, RawIFDEntry};
use byteorder::{ByteOrder, ReadBytesExt};
use failure::{Error, Fail};
use std::io::{Seek, SeekFrom};

/// IFD Entry Data, essentially a dynamic type for TIFF's tag values. All values are boxed slices,
/// as TIFF values are all arrays.
#[derive(Debug, Clone)]
enum IFDEntryData {
    Undefined(Box<[u8]>),
    Byte(Box<[u8]>),
    //Ascii(String),
    Ascii(Box<[String]>),
    Short(Box<[u16]>),
    Long(Box<[u32]>),
    Rational(Box<[(u32, u32)]>),
    /* Sbyte(Box<[i8]>),
    Sshort(Box<[i16]>),
    Slong(Box<[u32]>),
    SRational(Box<[(i32, i32)]>),
    Double(Box<[f64]>),
    Float(Box<[f32]>), */
    Unrecognized { tag_type: u16 },
}

use crate::constants::ifd_entry_type_magic::*;

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
        _ => false,
    }
}

/// Convert a TIFF Ascii sequence to Strings
fn tiff_ascii_to_strings<'a>(bytes: &'a [u8]) -> impl Iterator<Item = &'a str> {
    bytes
        .split(|x| *x == b'\0')
        .filter(|x| !x.is_empty())
        .map(|x| std::str::from_utf8(x).unwrap())
}

impl IFDEntryData {
    fn from_raw_entry<E: ByteOrder, R: ReadBytesExt + Seek>(
        reader: &mut R,
        entry: &RawIFDEntry,
    ) -> Result<Self, Error> {
        if tag_exceeds_ifd_field(entry.tag_type, entry.count) {
            let tag_data_offset = entry.value_or_offset.as_ref().read_u32::<E>()?;
            reader.seek(SeekFrom::Start(tag_data_offset.into()))?;
            Self::from_raw_entry_field::<E, R>(reader, entry.tag_type, entry.count as usize)
        } else {
            use std::io::Cursor;
            let mut tag_data_cursor = Cursor::new(entry.value_or_offset);
            Self::from_raw_entry_field::<E, _>(
                &mut tag_data_cursor,
                entry.tag_type,
                entry.count as usize,
            )
        }
    }

    fn from_raw_entry_field<E: ByteOrder, R: ReadBytesExt>(
        reader: &mut R,
        tag_type: u16,
        count: usize,
    ) -> Result<Self, Error> {
        Ok(match tag_type {
            IFD_TYPE_BYTE => {
                let mut buffer = vec![0; count];
                reader.read_exact(&mut buffer)?;
                IFDEntryData::Byte(buffer.into_boxed_slice())
            }
            IFD_TYPE_ASCII => {
                let mut buffer = vec![0; count];
                reader.read_exact(&mut buffer)?;
                IFDEntryData::Ascii(tiff_ascii_to_strings(&buffer).map(|string| String::from(string)).collect())
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
                let mut u32_buffer = vec![0; count * 2];
                reader.read_u32_into::<E>(&mut u32_buffer)?;
                let mut rational_buffer = Vec::with_capacity(count);
                for chunk in u32_buffer.chunks(2) {
                    rational_buffer.push((chunk[0], chunk[1]));
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
            other => {
                let mut buffer = vec![0; count];
                reader.read_exact(&mut buffer)?;
                IFDEntryData::Unrecognized {
                    tag_type,
                }
            }
        })
    }
}

#[derive(Debug, Clone)]
pub enum IFDEntry {
    Unknown { tag: u16, data: IFDEntryData },
}

impl IFDEntry {
    pub fn from_raw_entry<E: ByteOrder, R: ReadBytesExt + Seek>(
        reader: &mut R,
        entry: &RawIFDEntry,
    ) -> Result<Self, Error> {
        Ok(Self::Unknown {
            tag: entry.tag,
            data: IFDEntryData::from_raw_entry::<E, R>(reader, entry)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct IFD(Vec<IFDEntry>);

impl IFD {
    pub fn from_raw_ifd<E: ByteOrder, R: ReadBytesExt + Seek>(
        reader: &mut R,
        raw_ifd: &RawIFD,
    ) -> Result<Self, Error> {
        Ok(Self(
            raw_ifd
                .0
                .iter()
                .map(|entry| IFDEntry::from_raw_entry::<E, R>(reader, entry))
                .collect::<Result<Vec<IFDEntry>, Error>>()?,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct IFDTable(Vec<IFD>);

/*
impl IFDTable {
    pub fn from_raw_ifds<E: ByteOrder>(raw_ifds: Box<[RawIFD]>) -> Self {

    }
}
*/
