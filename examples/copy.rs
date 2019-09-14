use failure::{format_err, Fallible};
use tiff_concept::tags;
use tiff_concept::IFDEntryData;
use tiff_concept::{NativeEndian, TiffReader, TiffWriter};

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

    // Create reader and writer
    let mut reader = TiffReader::from_path(source_path)?;
    let mut writer = TiffWriter::<NativeEndian, _>::from_path(dest_path)?;

    let ifds = reader.ifds().cloned().collect::<Vec<_>>();

    for mut ifd in ifds {
        // Gather strip offsets and byte counts from the source image
        let (strip_offsets, strip_lengths) = match (
            ifd.get_tag(tags::STRIP_OFFSETS),
            ifd.get_tag(tags::STRIP_BYTE_COUNTS),
        ) {
            (Some(IFDEntryData::Long(off)), Some(IFDEntryData::Long(len))) => Ok((off, len)),
            (Some(_), None) | (None, Some(_)) | (None, None) => Err(format_err!("Missing tag")),
            (Some(_), Some(_)) => Err(format_err!("Tag is of wrong type")),
        }?;

        // Create buffers for strips and lengths produced by the writer
        let mut strip_offsets_out = Vec::new();
        let mut strip_lengths_out = Vec::new();

        // For each original strip offset and length, copy the data to the output image
        for (offset, length) in strip_offsets.iter().zip(strip_lengths.iter()) {
            let strip = reader.read_raw_strip(u64::from(*offset), u64::from(*length))?;
            let (offset, length) = writer.write_raw_strip(&strip)?;

            // Save the locations and lengths of the strips in the output image
            strip_offsets_out.push(offset as u32);
            strip_lengths_out.push(length as u32);
        }

        // Set the strip offsets and lengths on the output image (they are different from the original)
        *ifd.get_tag_mut(tags::STRIP_OFFSETS).unwrap() =
            IFDEntryData::Long(strip_offsets_out.into_boxed_slice());

        *ifd.get_tag_mut(tags::STRIP_BYTE_COUNTS).unwrap() =
            IFDEntryData::Long(strip_lengths_out.into_boxed_slice());

        // Write the modified IFD to output image
        writer.write_ifd(&ifd)?;
    }

    Ok(())
}
