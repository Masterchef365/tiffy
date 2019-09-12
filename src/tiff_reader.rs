use byteorder::{BigEndian, LittleEndian, ByteOrder, ReadBytesExt, WriteBytesExt};

use failure::Error;
use crate::header::*;
use crate::ifd::{IFDEntry, IFDEntryData, IFD};
use crate::raw_ifd::*;
use std::fs::File;
use std::io::BufReader;
use std::io::{Cursor, Seek, SeekFrom};

struct TiffReader<E: ByteOrder> {
    _phantomdata: std::marker::PhantomData<E>,
}

impl<E: ByteOrder> TiffReader<E> {
    pub fn new() -> Self {
        Self {
            _phantomdata: std::marker::PhantomData,
        }
    }
}

pub fn read_tiff() -> Box<TiffReader<dyn ByteOrder>> {
    Box::new(TiffReader::new::<LittleEndian>())
}

