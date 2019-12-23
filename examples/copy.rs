use failure::Fallible;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom};
use tiffy::baseline::tags;
use tiffy::lowlevel::{IFDField, MetadataReader, MetadataWriter, NativeEndian};

/// Rewrite (copy) an image's tags and data
fn main() -> Fallible<()> {
    // Parse arguments
    let mut args = std::env::args();
    let (source_path, dest_path) = match (args.next(), args.next(), args.next()) {
        (Some(_), Some(s), Some(d)) => (s, d),
        (Some(program_name), _, _) => {
            eprintln!("Usage: {} <source> <dest>", program_name);
            return Ok(());
        }
        _ => panic!("Program has no path"),
    };

    // Open files
    let mut source_file = BufReader::new(File::open(source_path)?);
    let mut dest_file = BufWriter::new(File::create(dest_path)?);

    // Create helpers
    let ifd_reader = MetadataReader::read_header(&mut source_file)?;
    let mut ifd_writer = MetadataWriter::<NativeEndian>::write_header(&mut dest_file)?;

    for ifd in ifd_reader.ifds() {
        // Gather strip offsets and byte counts from the source image
        let strip_offsets = ifd.get::<&[u32]>(tags::STRIP_OFFSETS)?;
        let strip_lengths = ifd.get::<&[u32]>(tags::STRIP_BYTE_COUNTS)?;

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

        let mut new_ifd = ifd.clone();

        // Careful about blindly copying tags between files, Unrecognized tags will write out the
        // _literal_ information from the tag instead of reordering it and setting a new pointer in
        // the file. This library _does_ give you enough rope to hang yourself if you so choose.
        new_ifd.entries.retain(|_, field| match field {
            IFDField::Unrecognized { .. } => false,
            _ => true,
        });

        // Set the strip offsets and lengths on the output image (they are different from the original)
        *new_ifd.entries.get_mut(&tags::STRIP_OFFSETS).unwrap() =
            IFDField::Long(strip_offsets_out.into_boxed_slice());

        *new_ifd.entries.get_mut(&tags::STRIP_BYTE_COUNTS).unwrap() =
            IFDField::Long(strip_lengths_out.into_boxed_slice());

        // Write the modified IFD to output image
        ifd_writer.write_ifd(&new_ifd, &mut dest_file)?;
    }

    Ok(())
}
