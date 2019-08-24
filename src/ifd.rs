use crate::constants::ifd_entry_type_magic::*;
use crate::raw_ifd::{RawIFD, RawIFDEntry};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use failure::Error;
use std::io::Cursor;
use std::io::{Seek, SeekFrom};

/// IFD Entry Data, essentially a dynamic type for TIFF's tag values. All values are boxed slices,
/// as TIFF values are all arrays.
#[derive(Debug, Clone)]
pub enum IFDEntryData {
    Undefined(Box<[u8]>),
    Byte(Box<[u8]>),
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
    /// The _type_ of the tag was unrecognized when reading
    Unrecognized {
        tag_type: u16,
    },
}

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
fn tiff_ascii_to_strings(bytes: &[u8]) -> impl Iterator<Item = &str> {
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
                IFDEntryData::Ascii(tiff_ascii_to_strings(&buffer).map(String::from).collect())
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
            _ => {
                let mut buffer = vec![0; count];
                reader.read_exact(&mut buffer)?;
                IFDEntryData::Unrecognized { tag_type }
            }
        })
    }

    pub fn write_into<E: ByteOrder, W: WriteBytesExt + Seek>(
        &self,
        writer: &mut W,
    ) -> Result<(), std::io::Error> {
        match self {
            Self::Undefined(bytes) => writer.write_all(&bytes),
            Self::Ascii(strings) => {
                for string in strings.iter() {
                    writer.write_all(string.as_bytes())?;
                    writer.write_all(&[0])?;
                }
                Ok(())
            }
            _ => unimplemented!(),
        }
    }

    pub fn get_type_and_count(&self) -> (u16, u32) {
        match self {
            Self::Undefined(data) => (IFD_TYPE_UNDEFINED, data.len() as u32),
            Self::Ascii(strings) => {
                let mut length: u32 = 0;
                for string in strings.iter() {
                    length += string.as_bytes().len() as u32;
                    length += 1; // For null character
                }
                (IFD_TYPE_ASCII, length)
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IFDEntry {
    pub tag: u16,
    pub data: IFDEntryData,
}

impl IFDEntry {
    pub fn from_raw_entry<E: ByteOrder, R: ReadBytesExt + Seek>(
        reader: &mut R,
        entry: &RawIFDEntry,
    ) -> Result<Self, Error> {
        Ok(Self {
            tag: entry.tag,
            data: IFDEntryData::from_raw_entry::<E, R>(reader, entry)?,
        })
    }

    pub fn to_raw_entry<E: ByteOrder, W: WriteBytesExt + Seek>(
        &self,
        writer: &mut W,
    ) -> Result<RawIFDEntry, Error> {
        let (tag_type, count) = self.data.get_type_and_count();

        let mut value_field = vec![];
        let mut cursor = Cursor::new(&mut value_field);

        if tag_exceeds_ifd_field(tag_type, count) {
            let data_offset = writer.seek(SeekFrom::Current(0))? as u32;
            cursor.write_u32::<E>(data_offset)?;
            self.data.write_into::<E, _>(writer)?;
        } else {
            self.data.write_into::<E, _>(&mut cursor)?;
        }

        let mut value_or_offset = [0; 4];
        value_or_offset.copy_from_slice(&value_field[0..4]);

        Ok(RawIFDEntry {
            tag: self.tag,
            tag_type,
            count,
            value_or_offset,
        })
    }
}

#[derive(Debug, Clone)]
pub struct IFD(pub Vec<IFDEntry>);

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

    pub fn to_raw_ifd<E: ByteOrder, W: WriteBytesExt + Seek>(
        &self,
        writer: &mut W,
    ) -> Result<RawIFD, Error> {
        Ok(RawIFD(
            self.0
                .iter()
                .map(|entry| entry.to_raw_entry::<E, W>(writer))
                .collect::<Result<Vec<RawIFDEntry>, Error>>()?,
        ))
    }
}
