#![allow(dead_code)]
/// Magic numbers
pub mod header_magic {
    pub const LITTLE_ENDIAN_MAGIC: [u8; 2] = [b'I', b'I'];
    pub const BIG_ENDIAN_MAGIC: [u8; 2] = [b'M', b'M'];
    pub const VERSION_MAGIC: u16 = 42;
}

/// IFD Field types
pub mod ifd_entry_type_magic {
    // Baseline
    pub const IFD_TYPE_BYTE: u16 = 1;
    pub const IFD_TYPE_ASCII: u16 = 2;
    pub const IFD_TYPE_SHORT: u16 = 3;
    pub const IFD_TYPE_LONG: u16 = 4;
    pub const IFD_TYPE_RATIONAL: u16 = 5;

    // Extended
    pub const IFD_TYPE_SBYTE: u16 = 6;
    pub const IFD_TYPE_UNDEFINED: u16 = 7;
    pub const IFD_TYPE_SSHORT: u16 = 8;
    pub const IFD_TYPE_SLONG: u16 = 9;
    pub const IFD_TYPE_SRATIONAL: u16 = 10;
    pub const IFD_TYPE_FLOAT: u16 = 11;
    pub const IFD_TYPE_DOUBLE: u16 = 12;
}

/// Compression magic numbers
pub mod compression_type_magic {
    pub const COMPRESSION_TYPE_UNCOMPRESSED: u16 = 1;
    pub const COMPRESSION_TYPE_CCITT_1D: u16 = 2;
    pub const COMPRESSION_TYPE_GROUP_3_FAX: u16 = 3;
    pub const COMPRESSION_TYPE_GROUP_4_FAX: u16 = 4;
    pub const COMPRESSION_TYPE_LZW: u16 = 5;
    pub const COMPRESSION_TYPE_JPEG: u16 = 6;
    pub const COMPRESSION_TYPE_NEW_JPEG: u16 = 7;
    pub const COMPRESSION_TYPE_ADOBE_DEFLATE: u16 = 8;
    pub const COMPRESSION_TYPE_JBIG_T85: u16 = 9;
    pub const COMPRESSION_TYPE_JBIG_T43: u16 = 0xA;
    pub const COMPRESSION_TYPE_NEXT: u16 = 0x7FFE;
    pub const COMPRESSION_TYPE_PACKBITS: u16 = 0x8005;
    pub const COMPRESSION_TYPE_THUNDERSCAN: u16 = 0x8029;
    pub const COMPRESSION_TYPE_RASTERPADDING: u16 = 0x807F;
    pub const COMPRESSION_TYPE_RLE_LINEWORK: u16 = 0x8080;
    pub const COMPRESSION_TYPE_RLE_HIGH_RES: u16 = 0x8081;
    pub const COMPRESSION_TYPE_RLE_BINARY_LINE: u16 = 0x8082;
    pub const COMPRESSION_TYPE_DEFLATE_PKZIP: u16 = 0x80B2;
    pub const COMPRESSION_TYPE_KODAK_DCS: u16 = 0x80B3;
    pub const COMPRESSION_TYPE_JBIG: u16 = 0x8765;
    pub const COMPRESSION_TYPE_JPEG2000: u16 = 0x8798;
    pub const COMPRESSION_TYPE_NIKON_NEF: u16 = 0x8799;
    pub const COMPRESSION_TYPE_JBIG2: u16 = 0x879B;
}

// Photometic Interpretation magic
pub mod photometic_interpretation_magic {
    pub const PHOTOMETRICINTERPRETATION_WHITEISZERO: u16 = 0;
    pub const PHOTOMETRICINTERPRETATION_BLACKISZERO: u16 = 1;
    pub const PHOTOMETRICINTERPRETATION_RGB: u16 = 2;
    pub const PHOTOMETRICINTERPRETATION_RGB_PALETTE: u16 = 3;
    pub const PHOTOMETRICINTERPRETATION_TRANSPARENCY_MASK: u16 = 4;
    pub const PHOTOMETRICINTERPRETATION_CMYK: u16 = 5;
    pub const PHOTOMETRICINTERPRETATION_YCBCR: u16 = 6;
    pub const PHOTOMETRICINTERPRETATION_CIELAB: u16 = 8;
}
