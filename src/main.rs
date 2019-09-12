#![allow(dead_code)]
mod tiff_reader;
mod constants;
mod header;
mod ifd;
mod raw_ifd;
mod tags;
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};
use failure::Error;
use header::*;
use ifd::{IFDEntry, IFDEntryData, IFD};
use raw_ifd::*;
use std::fs::File;
use std::io::BufReader;
use std::io::{Cursor, Seek, SeekFrom};

fn main() -> Result<(), Error> {
    let mut file = BufReader::new(File::open("/home/duncan/Downloads/Untitled.tiff")?);
    println!("{:?}", read_file(&mut file)?);
    //check_self::<BigEndian>()?;
    //check_self::<LittleEndian>()?;
    Ok(())
}

fn check_self<E: ByteOrder>() -> Result<(), Error> {
    let mut buffer = Cursor::new(Vec::new());

    // Write magic numbers
    write_header::<E, _>(&mut buffer)?;

    // Remember the position of the IFD pointer and leave it zero for now
    let first_ifd_pointer_position = buffer.seek(SeekFrom::Current(0))?;

    // Leave the IFD pointer at zero for now...
    buffer.write_u32::<E>(0)?;

    let ifd_table = vec![
        IFD(vec![
            IFDEntry {
                tag: 1337,
                data: IFDEntryData::Undefined(Box::new([0, 1, 2, 3])),
            },
            IFDEntry {
                tag: 3621,
                data: IFDEntryData::Ascii(Box::new([
                    "Test test".to_string(),
                    "Test test 2".to_string(),
                ])),
            },
        ]),
        IFD(vec![
            IFDEntry {
                tag: 3280,
                data: IFDEntryData::Rational(Box::new([(0, 1), (2, 3)])),
            },
            IFDEntry {
                tag: 3280,
                data: IFDEntryData::Long(Box::new([0, 1, 2, 3])),
            },
        ]),
    ];
    // Write image data here...

    // Write the long field values and create raw IFDs
    let mut raw_ifds = Vec::with_capacity(ifd_table.len());
    for ifd in &ifd_table {
        raw_ifds.push(ifd.to_raw_ifd::<E, _>(&mut buffer)?);
    }

    // Write the raw IFDs that refer to those values
    let ifd_table_position = buffer.seek(SeekFrom::Current(0))?;
    write_raw_ifds::<E, _>(&mut buffer, &raw_ifds)?;

    // Seek back to the IFD entry start and write the pointer
    buffer.seek(SeekFrom::Start(first_ifd_pointer_position))?;
    buffer.write_u32::<E>(ifd_table_position as u32)?;

    // Rewind and read for debugging
    buffer.seek(SeekFrom::Start(0))?;
    let read_back = read_file(&mut buffer)?;
    assert_eq!(read_back, ifd_table.into_boxed_slice());
    Ok(())
}

fn read_file<R: ReadBytesExt + Seek>(reader: &mut R) -> Result<Box<[IFD]>, Error> {
    if read_header_endian(reader)? {
        read_using_endian::<LittleEndian, _>(reader)
    } else {
        read_using_endian::<BigEndian, _>(reader)
    }
}

fn read_using_endian<E: ByteOrder, R: ReadBytesExt + Seek>(
    reader: &mut R,
) -> Result<Box<[IFD]>, Error> {
    read_header_magic::<R, E>(reader)?;
    let raw_ifds = read_raw_ifds::<E, _>(reader)?;
    let ifd_table = raw_ifds
        .iter()
        .map(|raw_ifd| IFD::from_raw_ifd::<E, _>(reader, &raw_ifd))
        .collect::<Result<Box<[_]>, _>>()?;
    Ok(ifd_table)
}
