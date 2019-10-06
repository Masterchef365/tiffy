use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::Error;

/// A struct representing a low-level IFD value.
#[derive(Debug, Clone, Copy)]
pub struct RawIFDField {
    /// Tag ID.
    pub tag: u16,

    /// Tag data type.
    pub tag_type: u16,

    /// Quantity (not byte count) of data in the field.
    pub count: u32,

    /// Field representing either the value of the tag (if it is small enough)
    /// or the file offset of the tag's data.
    pub value_or_offset: [u8; 4],
}

impl RawIFDField {
    /// Read the field value from `reader`.
    pub fn from_reader<E: ByteOrder, R: ReadBytesExt>(
        reader: &mut R,
    ) -> Result<Self, Error> {
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

    /// Write the field value to `writer`.
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

/// A struct representing a low-level IFD.
#[derive(Debug, Clone)]
pub struct RawIFD {
    pub entries: Vec<RawIFDField>,
}

impl RawIFD {
    /// Read an entire IFD from `reader` excluding the offset to the next IFD.
    pub fn from_reader<E: ByteOrder, R: ReadBytesExt>(reader: &mut R) -> Result<Self, Error> {
        // Read length header
        let field_count = reader.read_u16::<E>()? as usize;

        // Read entries
        let mut entries = Vec::with_capacity(field_count);
        for _ in 0..field_count {
            entries.push(RawIFDField::from_reader::<E, R>(reader)?);
        }
        Ok(Self { entries })
    }

    /// Write an entire IFD to `writer` excluding the offset to the next IFD.
    pub fn to_writer<E: ByteOrder, W: WriteBytesExt>(&self, writer: &mut W) -> Result<(), Error> {
        assert!(self.entries.len() < std::u16::MAX as usize);

        // Write length header
        writer.write_u16::<E>(self.entries.len() as u16)?;

        // Write entries
        for field in &self.entries {
            field.to_writer::<E, W>(writer)?;
        }
        Ok(())
    }
}
