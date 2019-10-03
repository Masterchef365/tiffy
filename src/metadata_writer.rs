use crate::header::write_header;
use crate::ifd::IFD;
use byteorder::{ByteOrder, WriteBytesExt};
use failure::Fallible;
use std::io::{Seek, SeekFrom};
use std::marker::PhantomData;

/// A TIFF metadata (header/IFD) writer.
pub struct MetadataWriter<E: ByteOrder> {
    /// File position of the last written IFD pointer
    last_ifd_pointer_position: u64,
    _phantomdata: PhantomData<E>,
}

impl<E: ByteOrder> MetadataWriter<E> {
    /// Create a MetadataWriter from `writer`. Note: Assumes the cursor is in a position ready for 
    /// writing the new file.
    pub fn write_header<W: WriteBytesExt + Seek>(writer: &mut W) -> Fallible<Self> {
        // Write the header
        write_header::<E, _>(writer)?;

        // Write zero for the first IFD pointer, and remember where you were
        let last_ifd_pointer_position = writer.seek(SeekFrom::Current(0))?;
        writer.write_u32::<E>(0)?;

        Ok(Self {
            last_ifd_pointer_position,
            _phantomdata: PhantomData,
        })
    }

    /// Write a single IFD (and its data) into the internal writer. Note: the cursor shall be
    /// advanced to a position after the data and IFD, ready for another write. Returns the
    /// position within the file of the beginning of the IFD just written.
    pub fn write_ifd<W: WriteBytesExt + Seek>(&mut self, ifd: &IFD, writer: &mut W) -> Fallible<u64> {
        // Write out the fields
        let raw_ifd = ifd.write_fields_to::<E, _>(writer)?;

        // Save the current cursor position as it will become the pointer to the next IFD
        let ifd_table_position = writer.seek(SeekFrom::Current(0))?;

        // Write the IFD into the file
        raw_ifd.to_writer::<E, _>(writer)?;

        // Create a pointer to the 'next IFD' pointer
        let next_ifd_table_pointer_position = writer.seek(SeekFrom::Current(0))?;

        // Write zero to that pointer for now
        writer.write_u32::<E>(0)?;

        // Save the position after the end of the table to restore it so this function seems to
        // write only the table and data sequentially
        let position_after_table = writer.seek(SeekFrom::Current(0))?;

        // Seek to the last pointer
        let _ = writer.seek(SeekFrom::Start(self.last_ifd_pointer_position));

        // Write the position of the IFD we just wrote to it
        writer.write_u32::<E>(ifd_table_position as u32)?;
        
        // Save the pointer to the 'next IFD' in our struct
        self.last_ifd_pointer_position = next_ifd_table_pointer_position;

        // Return to the position after the IFD
        let _ = writer.seek(SeekFrom::Start(position_after_table))?;

        Ok(ifd_table_position)
    }
}
