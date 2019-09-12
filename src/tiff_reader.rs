use crate::header::*;
use crate::ifd::IFD;
use crate::raw_ifd::*;
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use failure::Fallible;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::path::Path;

pub struct TiffReader<R: ReadBytesExt + Seek> {
    is_little_endian: bool,
    ifd_table: Box<[IFD]>,
    reader: R,
}

impl TiffReader<BufReader<File>> {
    pub fn from_path(path: impl AsRef<Path>) -> Fallible<Self> {
        Self::from_reader(BufReader::new(File::open(path)?))
    }
}

impl<R: ReadBytesExt + Seek> TiffReader<R> {
    /// Create a new TiffReader from `reader`
    pub fn from_reader(mut reader: R) -> Fallible<Self> {
        // Write headers
        let is_little_endian = read_header_endian(&mut reader)?;

        let ifd_table = if is_little_endian {
            Self::read_ifd_table_endian::<LittleEndian>(&mut reader)?
        } else {
            Self::read_ifd_table_endian::<BigEndian>(&mut reader)?
        };

        Ok(Self {
            is_little_endian,
            ifd_table,
            reader,
        })
    }

    /// ReadBytesExt all of the IFDs with the specified endian
    fn read_ifd_table_endian<E: ByteOrder>(reader: &mut R) -> Fallible<Box<[IFD]>> {
        read_header_magic::<E, _>(reader)?;
        let raw_ifds = Self::read_raw_ifds::<E>(reader)?;
        let mut ifds = Vec::with_capacity(raw_ifds.len());
        for raw_ifd in raw_ifds.iter() {
            ifds.push(IFD::read_fields_from::<E, _>(reader, &raw_ifd)?);
        }
        Ok(ifds.into_boxed_slice())
    }

    /// ReadBytesExt all IFDs from `reader` table into memory sequentially
    fn read_raw_ifds<E: ByteOrder>(reader: &mut R) -> Fallible<Box<[RawIFD]>> {
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

    /// Returns true if the file is in little-endian byte order
    pub fn is_little_endian(&self) -> bool {
        self.is_little_endian
    }

    /// Returns an iterator over references to this file's IFDs
    pub fn ifds(&self) -> impl Iterator<Item = &IFD> {
        self.ifd_table.iter()
    }
}
