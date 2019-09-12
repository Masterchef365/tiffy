use failure::{format_err, Error, Fallible};
use std::io::{Seek, SeekFrom};
use tiff_concept::tags;
use tiff_concept::IFDEntryData;
use tiff_concept::{NativeEndian, TiffReader, TiffWriter};

fn main() -> Fallible<()> {
    let mut reader = TiffReader::from_path("/home/duncan/Downloads/test.tiff")?;
    let mut writer = TiffWriter::<NativeEndian, _>::from_path("/home/duncan/wat.tiff")?;
    let mut ifds = reader.ifds().cloned().collect::<Vec<_>>();

    for ifd in ifds {
        let (strip_offsets, strip_lengths) = match (
            ifd.get_tag(tags::STRIP_OFFSETS),
            ifd.get_tag(tags::STRIP_BYTE_COUNTS),
        ) {
            (Some(IFDEntryData::Long(o)), Some(IFDEntryData::Long(l))) => (o, l),
            other => return Err(format_err!("Got {:?}", other)),
        };

        let strips = strip_offsets
            .iter()
            .zip(strip_lengths.iter())
            .map(|(offset, length)| reader.read_strip(*offset as u64, *length as u64))
            .collect::<Fallible<Vec<_>>>()?;

        let mut new_ifd = ifd.clone();
        let strip_positions = strips
            .iter()
            .map(|strip| writer.write_strip(strip))
            .collect::<Fallible<Vec<_>>>()?;

        *new_ifd.get_tag_mut(tags::STRIP_OFFSETS).unwrap() = IFDEntryData::Long(
            strip_positions
                .iter()
                .map(|(p, _)| *p as u32)
                .collect::<Box<[_]>>(),
        );

        *new_ifd.get_tag_mut(tags::STRIP_BYTE_COUNTS).unwrap() = IFDEntryData::Long(
            strip_positions
                .iter()
                .map(|(_, p)| *p as u32)
                .collect::<Box<[_]>>(),
        );
        writer.write_ifd(&new_ifd)?;
    }
    Ok(())
}

/*
use std::io::{Cursor, Seek, SeekFrom};

fn main() -> Result<(), Error> {
    //let mut file = BufReader::new(File::open("/home/duncan/Downloads/Untitled.tiff")?);
    //println!("{:?}", read_file(&mut file)?);
    check_self::<BigEndian>()?;
    check_self::<LittleEndian>()?;
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
*/
