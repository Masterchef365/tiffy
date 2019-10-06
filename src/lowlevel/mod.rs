/// Conversions between some primitive types and IFDFieldData
pub(crate) mod field_conversions;
pub use field_conversions::*;

/// TIFF metadata reader
pub(crate) mod metadata_reader;
pub use metadata_reader::*;

/// TIFF metadata writer
pub(crate) mod metadata_writer;
pub use metadata_writer::*;

/// Non-tag magic numbers
pub mod constants;

/// Header reading/writing
pub(crate) mod header;
pub use header::*;

/// High-level IFD abstrations
pub(crate) mod ifd;
pub(crate) mod ifd_field;
pub use ifd_field::*;
pub use ifd::*;

/// RawIFDs are the non-dereferenced low-level versions of their high-level counterparts -
/// they usually only contain the pointers to other data within the TIFF file.
pub(crate) mod raw_ifd;
pub use raw_ifd::*;

/// Byteorder
pub use byteorder::{LittleEndian, BigEndian, NativeEndian};
