use crate::constants::header_magic::VERSION_MAGIC;
use byteorder::{ByteOrder, ReadBytesExt};
use failure::{Error, Fail};
use std::io::{Seek, SeekFrom};

#[derive(Debug, Clone, Copy)]
pub struct RawIFDEntry {
    pub tag: u16,
    pub tag_type: u16,
    pub count: u32,
    pub value_or_offset: [u8; 4],
}

impl RawIFDEntry {
    pub fn from_reader<E: ByteOrder, R: ReadBytesExt>(
        reader: &mut R,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            tag: reader.read_u16::<E>()?,
            tag_type: reader.read_u16::<E>()?,
            count: reader.read_u32::<E>()?,
            value_or_offset: {
                let mut buffer = [0; 4];
                reader.read_exact(&mut buffer)?;
                buffer
            },
        })
    }
}

#[derive(Debug, Clone)]
pub struct RawIFD(pub Vec<RawIFDEntry>);

impl RawIFD {
    pub fn from_reader<E: ByteOrder, R: ReadBytesExt>(
        reader: &mut R,
    ) -> Result<Self, std::io::Error> {
        let entry_count = reader.read_u16::<E>()? as usize;
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            entries.push(RawIFDEntry::from_reader::<E, R>(reader)?);
        }
        Ok(Self(entries))
    }
}

#[derive(Fail, Debug)]
enum IFDTableReadError {
    #[fail(display = "Bad magic number: {:?}", magic)]
    BadMagic { magic: u16 },
}

pub fn read_raw_ifds<E: ByteOrder, R: ReadBytesExt + Seek>(
    reader: &mut R,
) -> Result<Box<[RawIFD]>, Error> {
    let magic = reader.read_u16::<E>()?;
    let mut ifds = Vec::new();
    if magic != VERSION_MAGIC {
        return Err(IFDTableReadError::BadMagic { magic }.into());
    }
    'ifd_load: loop {
        let next_ifd_offset = reader.read_u32::<E>()?;
        if next_ifd_offset == 0 {
            break 'ifd_load;
        }
        reader.seek(SeekFrom::Start(next_ifd_offset.into()))?;
        ifds.push(RawIFD::from_reader::<E, R>(reader)?);
    }
    Ok(ifds.into_boxed_slice())
}
