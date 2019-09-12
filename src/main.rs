use failure::{format_err, Fallible};
use tiff_concept::tags;
use tiff_concept::IFDEntryData;
use tiff_concept::{NativeEndian, TiffReader, TiffWriter};

fn main() -> Fallible<()> {
    let mut reader = TiffReader::from_path("/home/duncan/Downloads/test.tiff")?;
    let mut writer = TiffWriter::<NativeEndian, _>::from_path("/home/duncan/wat.tiff")?;
    let ifds = reader.ifds().cloned().collect::<Vec<_>>();

    for ifd in ifds {
        let (strip_offsets, strip_lengths) = match (
            ifd.get_tag(tags::STRIP_OFFSETS),
            ifd.get_tag(tags::STRIP_BYTE_COUNTS),
        ) {
            (Some(IFDEntryData::Long(off)), Some(IFDEntryData::Long(len))) => Ok((off, len)),
            (Some(_), None) | (None, Some(_)) | (None, None) => Err(format_err!("Missing tag")),
            (Some(_), Some(_)) => Err(format_err!("Tag is of wrong type")),
        }?;

        let mut new_ifd = ifd.clone();

        let mut strip_offsets_out = Vec::new();
        let mut strip_lengths_out = Vec::new();

        for (offset, length) in strip_offsets.iter().zip(strip_lengths.iter()) {
            let strip = reader.read_raw_strip(u64::from(*offset), u64::from(*length))?;
            let (offset, length) = writer.write_raw_strip(&strip)?;
            strip_offsets_out.push(offset as u32);
            strip_lengths_out.push(length as u32);
        }

        *new_ifd.get_tag_mut(tags::STRIP_OFFSETS).unwrap() =
            IFDEntryData::Long(strip_offsets_out.into_boxed_slice());

        *new_ifd.get_tag_mut(tags::STRIP_BYTE_COUNTS).unwrap() =
            IFDEntryData::Long(strip_lengths_out.into_boxed_slice());

        writer.write_ifd(&new_ifd)?;
    }
    Ok(())
}
