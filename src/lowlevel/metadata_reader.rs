use crate::lowlevel::{
    header::{read_header_endian, read_header_magic},
    ifd::IFD,
    raw_ifd::*,
};
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use failure::Fallible;
use std::io::{Seek, SeekFrom};

/// A TIFF metadata (header/IFD) reader.
pub struct MetadataReader {
    /// Whether the file is in little endian byte order.
    is_little_endian: bool,
    /// Table of IFDs read.
    ifd_table: Box<[IFD]>,
}

impl MetadataReader {
    /// Create a new MetadataReader from `reader`, reading the entire IFD table from the file.
    /// Assumes the cursor is positioned at a 32-bit pointer to the first valid IFD.
    pub fn read_header<R: ReadBytesExt + Seek>(reader: &mut R) -> Fallible<Self> {
        let is_little_endian = read_header_endian(reader)?;

        let ifd_table = if is_little_endian {
            read_ifd_table_endian::<LittleEndian, R>(reader)?
        } else {
            read_ifd_table_endian::<BigEndian, R>(reader)?
        };

        Ok(Self {
            is_little_endian,
            ifd_table,
        })
    }

    /// Read the IFD table starting at `offset` within the reader (For use with sub-IFDs for example).
    pub fn read_external_ifd_table<R: ReadBytesExt + Seek>(
        &mut self,
        offset: u64,
        reader: &mut R,
    ) -> Fallible<Box<[IFD]>> {
        reader.seek(SeekFrom::Start(offset))?;
        if self.is_little_endian {
            read_ifd_table_endian::<LittleEndian, R>(reader)
        } else {
            read_ifd_table_endian::<BigEndian, R>(reader)
        }
    }

    /// Returns true if the file is in little-endian byte order.
    pub fn is_little_endian(&self) -> bool {
        self.is_little_endian
    }

    /// Returns an iterator over references to this file's IFDs in the order they were read.
    pub fn ifds(&self) -> impl Iterator<Item = &IFD> {
        self.ifd_table.iter()
    }
}

/// Read all of the IFDs with the specified endian.
/// Assumes the cursor is positioned at the beginning of a TIFF file.
pub fn read_ifd_table_endian<E: ByteOrder, R: ReadBytesExt + Seek>(
    reader: &mut R,
) -> Fallible<Box<[IFD]>> {
    read_header_magic::<E, _>(reader)?;
    let raw_ifds = read_raw_ifds::<E, R>(reader)?;
    let mut ifds = Vec::with_capacity(raw_ifds.len());
    for raw_ifd in raw_ifds.iter() {
        ifds.push(IFD::read_from::<E, _>(reader, &raw_ifd)?);
    }
    Ok(ifds.into_boxed_slice())
}

/// Read all IFDs from `reader` table into memory sequentially.
/// Assumes the cursor is positioned at a 32-bit pointer to the first valid IFD.
pub fn read_raw_ifds<E: ByteOrder, R: ReadBytesExt + Seek>(
    reader: &mut R,
) -> Fallible<Box<[RawIFD]>> {
    let mut ifds = Vec::new();
    loop {
        let next_ifd_offset = reader.read_u32::<E>()?;
        if next_ifd_offset == 0 {
            break;
        }
        reader.seek(SeekFrom::Start(next_ifd_offset.into()))?;
        ifds.push(RawIFD::from_reader::<E, R>(reader)?);
    }
    Ok(ifds.into_boxed_slice())
}
