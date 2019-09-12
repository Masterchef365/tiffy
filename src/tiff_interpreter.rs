use crate::tiff_reader::TiffReader;
use byteorder::ReadBytesExt;
use std::io::Seek;

pub struct TiffInterpreter<R: Seek + ReadBytesExt> {
    reader: TiffReader<R>,
}

impl<R: Seek + ReadBytesExt> TiffInterpreter<R> {
    pub fn from_tiffreader(reader: TiffReader<R>) -> Self {
        Self { reader }
    }
}
