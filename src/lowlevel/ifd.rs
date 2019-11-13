use crate::lowlevel::ifd_field::IFDField;
use crate::lowlevel::raw_ifd::{RawIFD, RawIFDField};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;
use std::io::{self, Seek};

/// A high-level representation of an Image File Directory.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IFD {
    pub entries: HashMap<u16, IFDField>,
}

impl IFD {
    /// Create an empty IFD.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Read the fields from `reader` into memory, (de)referencing information from `raw_ifd`.
    pub fn read_from<E: ByteOrder, R: ReadBytesExt + Seek>(
        reader: &mut R,
        raw_ifd: &RawIFD,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            entries: raw_ifd
                .entries
                .iter()
                .map(|field| {
                    IFDField::read_from::<E, R>(reader, field).map(|data| (field.tag, data))
                })
                .collect::<Result<HashMap<u16, IFDField>, io::Error>>()?,
        })
    }

    /// Write the fields into `writer`, returning a RawIFD describing their locations and/or data.
    pub fn write_to<E: ByteOrder, W: WriteBytesExt + Seek>(
        &self,
        writer: &mut W,
    ) -> Result<RawIFD, io::Error> {
        // Tags must be sorted in ascending order
        let mut sorted_entries = self.entries.iter().collect::<Vec<(&u16, &IFDField)>>();
        sorted_entries.sort_by_key(|(tag, _)| *tag);
        Ok(RawIFD {
            entries: sorted_entries
                .iter()
                .map(|(&tag, data)| data.write_to::<E, W>(writer, tag))
                .collect::<Result<Vec<RawIFDField>, io::Error>>()?,
        })
    }
}
