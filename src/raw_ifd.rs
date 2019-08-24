use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use failure::Error;
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

    pub fn to_writer<E: ByteOrder, W: WriteBytesExt>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        writer.write_u16::<E>(self.tag)?;
        writer.write_u16::<E>(self.tag_type)?;
        writer.write_u32::<E>(self.count)?;
        writer.write_all(&self.value_or_offset)?;
        Ok(())
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

    pub fn to_writer<E: ByteOrder, W: WriteBytesExt>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        assert!(self.0.len() < std::u16::MAX as usize);
        writer.write_u16::<E>(self.0.len() as u16)?;
        for entry in &self.0 {
            entry.to_writer::<E, W>(writer)?;
        }
        Ok(())
    }
}

pub fn read_raw_ifds<E: ByteOrder, R: ReadBytesExt + Seek>(
    reader: &mut R,
) -> Result<Box<[RawIFD]>, Error> {
    let mut ifds = Vec::new();
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
