/// Raw IFDs are the disk-stored versions of their counterparts -
/// they usually only contain the data necessary to point to other sources of data.
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use failure::Error;
use std::io::{Seek, SeekFrom};

/// A struct representing a disk-stored IFD value.
#[derive(Debug, Clone, Copy)]
pub struct RawIFDEntry {
    pub tag: u16,
    pub tag_type: u16,
    pub count: u32,
    pub value_or_offset: [u8; 4],
}

impl RawIFDEntry {
    /// Read the entry value from `reader`.
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

    /// Write the entry value to `writer`.
    pub fn to_writer<E: ByteOrder, W: WriteBytesExt>(
        &self,
        writer: &mut W,
    ) -> Result<(), std::io::Error> {
        writer.write_u16::<E>(self.tag)?;
        writer.write_u16::<E>(self.tag_type)?;
        writer.write_u32::<E>(self.count)?;
        writer.write_all(&self.value_or_offset)?;
        Ok(())
    }
}

/// A struct representing a disk-stored IFD.
#[derive(Debug, Clone)]
pub struct RawIFD {
    pub entries: Vec<RawIFDEntry>,
}

impl RawIFD {
    /// Read an entire IFD from `reader`
    pub fn from_reader<E: ByteOrder, R: ReadBytesExt>(reader: &mut R) -> Result<Self, Error> {
        // Read length header
        let entry_count = reader.read_u16::<E>()? as usize;

        // Read entries
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            entries.push(RawIFDEntry::from_reader::<E, R>(reader)?);
        }
        Ok(Self { entries })
    }

    /// Write an entire IFD to `writer`
    pub fn to_writer<E: ByteOrder, W: WriteBytesExt>(&self, writer: &mut W) -> Result<(), Error> {
        assert!(self.entries.len() < std::u16::MAX as usize);

        // Write length header
        writer.write_u16::<E>(self.entries.len() as u16)?;

        // Write entries
        for entry in &self.entries {
            entry.to_writer::<E, W>(writer)?;
        }
        Ok(())
    }
}
