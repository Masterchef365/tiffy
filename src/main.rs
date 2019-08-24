#![allow(dead_code)]
mod constants;
mod header;
mod ifd;
mod raw_ifd;
mod tags;
use byteorder::{BigEndian, ByteOrder, LittleEndian, NativeEndian, ReadBytesExt, WriteBytesExt};
use failure::Error;
use header::*;
use ifd::{IFDEntry, IFDEntryData, IFD};
use raw_ifd::*;
use std::io::{Cursor, Seek, SeekFrom};
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn main() -> Result<(), Error> {
    let mut file = BufReader::new(File::open("/home/duncan/Untitled.tiff")?);
    println!("{:?}", read_file(&mut file)?);
    Ok(())
    //check_self::<NativeEndian>()
}

fn check_self<E: ByteOrder>() -> Result<(), Error> {
    let mut buffer = Cursor::new(Vec::new());

    // Write magic numbers
    header_magic_to_writer::<E, _>(&mut buffer)?;

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
    println!("{:?}", ifd_table);

    // Write image data here...

    // Write the long field values and create raw IFDs
    let mut raw_ifds = Vec::with_capacity(ifd_table.len());
    for ifd in ifd_table {
        raw_ifds.push(ifd.to_raw_ifd::<E, _>(&mut buffer)?);
    }

    // Write the raw IFDs that refer to those values
    let ifd_table_position = buffer.seek(SeekFrom::Current(0))?;
    write_raw_ifds::<E, _>(&mut buffer, raw_ifds.into_boxed_slice())?;

    // Seek back to the IFD entry start and write the pointer
    buffer.seek(SeekFrom::Start(first_ifd_pointer_position))?;
    buffer.write_u32::<E>(ifd_table_position as u32)?;

    // Rewind and read for debugging
    buffer.seek(SeekFrom::Start(0))?;
    println!("{:?}", read_file(&mut buffer)?);
    Ok(())
}

fn read_file<R: ReadBytesExt + Seek>(reader: &mut R) -> Result<Box<[IFD]>, Error> {
    if header_endian_is_little(reader)? {
        read_using_endian::<LittleEndian, _>(reader)
    } else {
        read_using_endian::<BigEndian, _>(reader)
    }
}

fn read_using_endian<E: ByteOrder, R: ReadBytesExt + Seek>(
    reader: &mut R,
) -> Result<Box<[IFD]>, Error> {
    check_magic::<R, E>(reader)?;
    let raw_ifds = read_raw_ifds::<E, _>(reader)?;
    let ifd_table = raw_ifds
        .iter()
        .map(|raw_ifd| IFD::from_raw_ifd::<E, _>(reader, &raw_ifd))
        .collect::<Result<Box<[_]>, _>>()?;
    Ok(ifd_table)
}
