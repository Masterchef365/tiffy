use failure::{format_err, Fallible};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write, Seek, SeekFrom};
use tiffy::{
    tags, IFDFieldData, {IFDReader, IFDWriter, NativeEndian},
};

/// Rewrite (copy) an image's tags and data
fn main() -> Fallible<()> {
    // Parse arguments
    let mut args = std::env::args();
    let (source_path, dest_path) = match (args.next(), args.next(), args.next()) {
        (Some(_), Some(s), Some(d)) => (s, d),
        (Some(program_name), _, _) => {
            eprintln!("Usage: {} <source> <dest>", program_name);
            std::process::exit(-1)
        }
        _ => panic!("Program has no path"),
    };

    // Create ifd_reader and ifd_writer
    let mut source_file = BufReader::new(File::open(source_path)?);
    let mut dest_file = BufWriter::new(File::create(dest_path)?);
    let ifd_reader = IFDReader::from_reader(&mut source_file)?;
    let mut ifd_writer = IFDWriter::<NativeEndian>::new_header(&mut dest_file)?;

    let ifds = ifd_reader.ifds().cloned().collect::<Vec<_>>();

    for mut ifd in ifds {
        // Gather strip offsets and byte counts from the source image
        let (strip_offsets, strip_lengths) = match (
            ifd.get_tag(tags::STRIP_OFFSETS),
            ifd.get_tag(tags::STRIP_BYTE_COUNTS),
        ) {
            (Some(IFDFieldData::Long(off)), Some(IFDFieldData::Long(len))) => Ok((off, len)),
            (Some(_), None) | (None, Some(_)) | (None, None) => Err(format_err!("Missing tag")),
            (Some(_), Some(_)) => Err(format_err!("Tag is of wrong type")),
        }?;

        // Create buffers for strips and lengths produced by the ifd_writer
        let mut strip_offsets_out = Vec::new();
        let mut strip_lengths_out = Vec::new();

        // For each original strip offset and length, copy the data to the output image
        for (offset, length) in strip_offsets.iter().zip(strip_lengths.iter()) {
            source_file.seek(SeekFrom::Start(*offset as u64))?;
            let strip_offset = dest_file.seek(SeekFrom::Current(0))? as u32;
            std::io::copy(
                &mut source_file.by_ref().take(*length as u64),
                &mut dest_file,
            )?;

            // Save the locations and lengths of the strips in the output image
            strip_offsets_out.push(strip_offset);
            strip_lengths_out.push(*length);
        }

        // Set the strip offsets and lengths on the output image (they are different from the original)
        *ifd.get_tag_mut(tags::STRIP_OFFSETS).unwrap() =
            IFDFieldData::Long(strip_offsets_out.into_boxed_slice());

        *ifd.get_tag_mut(tags::STRIP_BYTE_COUNTS).unwrap() =
            IFDFieldData::Long(strip_lengths_out.into_boxed_slice());

        // Write the modified IFD to output image
        ifd_writer.write_ifd(&ifd, &mut dest_file)?;
    }
    dest_file.flush()?;

    Ok(())
}
