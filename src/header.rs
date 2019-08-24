use crate::constants::header_magic::{BIG_ENDIAN_MAGIC, LITTLE_ENDIAN_MAGIC, VERSION_MAGIC};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use failure::{Error, Fail};
use std::io::Write;

#[derive(Fail, Debug)]
enum HeaderError {
    #[fail(display = "Bad endian magic number: {:?}", culprit)]
    BadEndianMagic { culprit: [u8; 2] },
    #[fail(display = "Bad magic number: {:?}", magic)]
    BadMagic { magic: u16 },
}

pub fn header_endian_is_little<R: ReadBytesExt>(reader: &mut R) -> Result<bool, Error> {
    let mut endian_magic = [0u8; 2];
    reader.read_exact(&mut endian_magic)?;
    match endian_magic {
        LITTLE_ENDIAN_MAGIC => Ok(true),
        BIG_ENDIAN_MAGIC => Ok(false),
        culprit => Err(HeaderError::BadEndianMagic { culprit }.into()),
    }
}

/// Check the magic number from this writer
pub fn check_magic<R: ReadBytesExt, E: ByteOrder>(reader: &mut R) -> Result<(), Error> {
    let magic = reader.read_u16::<E>()?;
    if magic != VERSION_MAGIC {
        Err(HeaderError::BadMagic { magic }.into())
    } else {
        Ok(())
    }
}

/// Little hack to determine endianness at runtime from Endian type 
fn endian_type_is_little<E: ByteOrder>() -> bool {
    E::read_u16(&[42, 0]) == 42
}

/// Write the header and magic number to a writer
pub fn header_magic_to_writer<E: ByteOrder, W: Write>(
    writer: &mut W,
) -> Result<(), std::io::Error> {
    if endian_type_is_little::<E>() {
        writer.write(&LITTLE_ENDIAN_MAGIC)
    } else {
        writer.write(&BIG_ENDIAN_MAGIC)
    }?;
    writer.write_u16::<E>(VERSION_MAGIC)
}
