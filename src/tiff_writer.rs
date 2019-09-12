use crate::header::*;
use crate::ifd::IFD;
use crate::raw_ifd::*;
use byteorder::{ByteOrder, WriteBytesExt};
use failure::Fallible;
use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::path::Path;

pub struct TiffWriter<E: ByteOrder, W: Write + Seek> {
    writer: W,
    last_ifd_pointer_position: u64,
    _phantomdata: PhantomData<E>,
}

impl<E: ByteOrder> TiffWriter<E, BufWriter<File>> {
    pub fn from_path(path: impl AsRef<Path>) -> Fallible<Self> {
        Self::from_writer(BufWriter::new(File::create(path)?))
    }
}

impl<E: ByteOrder, W: Write + Seek> TiffWriter<E, W> {
    pub fn from_writer(mut writer: W) -> Fallible<Self> {
        // Write the header
        write_header::<E, _>(&mut writer)?;

        // Write zero for the first IFD pointer, and remember where you were
        let last_ifd_pointer_position = writer.seek(SeekFrom::Current(0))?;
        writer.write_u32::<E>(0)?;

        Ok(Self {
            writer,
            last_ifd_pointer_position,
            _phantomdata: PhantomData,
        })
    }

    pub fn write_ifd(&mut self, ifd: &IFD) -> Fallible<()> {
    //pub(crate) fn write_ifd(&mut self, ifd: IFD) -> Fallible<()> {
        // Write out the fields
        let raw_ifd = ifd.write_fields_to::<E, _>(&mut self.writer)?;

        // Save the current cursor position as it will become the pointer to the next IFD
        let ifd_table_position = self.writer.seek(SeekFrom::Current(0))?;

        // Write the IFD into the file
        raw_ifd.to_writer::<E, _>(&mut self.writer)?;

        // Create a pointer to the 'next IFD' pointer
        let next_ifd_table_pointer_position = self.writer.seek(SeekFrom::Current(0))?;

        // Write zero to that pointer for now
        self.writer.write_u32::<E>(0)?;

        // Seek to the last pointer
        let _ = self.writer.seek(SeekFrom::Start(self.last_ifd_pointer_position));

        // Write the position of the IFD we just wrote to it
        self.writer.write_u32::<E>(ifd_table_position as u32)?;
        
        // Save the pointer to the 'next IFD' in our struct
        self.last_ifd_pointer_position = next_ifd_table_pointer_position;

        Ok(())
    }
}
