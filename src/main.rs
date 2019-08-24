#![allow(dead_code)]
mod constants;
mod header;
mod ifd;
mod raw_ifd;
mod tags;
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use failure::Error;
use header::header_endian_is_little;
use ifd::IFD;
use raw_ifd::read_raw_ifds;
use std::io::Seek;

fn main() -> Result<(), Error> {
    use std::fs::File;
    use std::io::BufReader;
    let mut file = BufReader::new(File::open("/home/duncan/Untitled.tiff")?);
    if header_endian_is_little(&mut file)? {
        run_endian::<LittleEndian, _>(&mut file)
    } else {
        run_endian::<BigEndian, _>(&mut file)
    }
}

fn run_endian<E: ByteOrder, R: ReadBytesExt + Seek>(reader: &mut R) -> Result<(), Error> {
    let raw_ifds = read_raw_ifds::<E, _>(reader)?;
    for raw_ifd in raw_ifds.iter() {
        println!("{:#?}", IFD::from_raw_ifd::<E, _>(reader, raw_ifd));
    }
    Ok(())
}
