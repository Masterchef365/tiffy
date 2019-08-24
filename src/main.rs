#![allow(dead_code)]
mod constants;
mod header;
mod ifd;
mod raw_ifd;
mod tags;
use byteorder::{BigEndian, ByteOrder, LittleEndian, NativeEndian, ReadBytesExt, WriteBytesExt};
use failure::Error;
use header::*;
use raw_ifd::{read_raw_ifds, RawIFD, RawIFDEntry};
use std::io::{Cursor, Seek, SeekFrom};
//use ifd::IFD;
//use std::fs::File;
//use std::io::{BufReader, BufWriter};

fn main() -> Result<(), Error> {
    //let mut file = BufReader::new(File::open("/home/duncan/Untitled.tiff")?);
    //println!("{:?}", read_file(&mut file)?);
    println!("{:?}", check_self::<NativeEndian>()?);
    Ok(())
}

fn check_self<E: ByteOrder>() -> Result<Box<[RawIFD]>, Error> {
    let mut buffer = Cursor::new(Vec::new());
    header_magic_to_writer::<E, _>(&mut buffer)?;
    let pos = buffer.seek(SeekFrom::Current(0))?;
    buffer.write_u32::<E>(pos as u32 + 4)?;
    RawIFD(vec![RawIFDEntry {
        tag: 1337,
        tag_type: 1,
        count: 1,
        value_or_offset: [0, 0, 0, 1],
    }])
    .to_writer::<E, _>(&mut buffer)?;
    buffer.write_u32::<E>(0)?;

    buffer.seek(SeekFrom::Start(0))?;

    read_file(&mut buffer)
}

fn read_file<R: ReadBytesExt + Seek>(reader: &mut R) -> Result<Box<[RawIFD]>, Error> {
    if header_endian_is_little(reader)? {
        read_using_endian::<LittleEndian, _>(reader)
    } else {
        read_using_endian::<BigEndian, _>(reader)
    }
}

fn read_using_endian<E: ByteOrder, R: ReadBytesExt + Seek>(
    reader: &mut R,
) -> Result<Box<[RawIFD]>, Error> {
    check_magic::<R, E>(reader)?;
    read_raw_ifds::<E, _>(reader)
}
