use byteorder::ReadBytesExt;
use failure::{Fail, Error};
#[derive(Fail, Debug)]
enum HeaderError {
    #[fail(display = "Bad endian magic number: {:?}", culprit)]
    BadEndianMagic { culprit: [u8; 2] },
}

use crate::constants::header_magic::{BIG_ENDIAN_MAGIC, LITTLE_ENDIAN_MAGIC};
pub fn header_endian_is_little<R: ReadBytesExt>(reader: &mut R) -> Result<bool, Error> {
    let mut endian_magic = [0u8; 2];
    reader.read_exact(&mut endian_magic)?;
    match endian_magic {
        LITTLE_ENDIAN_MAGIC => Ok(true),
        BIG_ENDIAN_MAGIC => Ok(false),
        culprit => Err(HeaderError::BadEndianMagic { culprit }.into()),
    }
}

