/// Compression magic numbers
pub mod compression_type {
    pub const UNCOMPRESSED: u16 = 0x0001;
    pub const CCITT_1D: u16 = 0x0002;
    pub const GROUP_3_FAX: u16 = 0x0003;
    pub const GROUP_4_FAX: u16 = 0x0004;
    pub const LZW: u16 = 0x0005;
    pub const JPEG: u16 = 0x0006;
    pub const NEW_JPEG: u16 = 0x0007;
    pub const ADOBE_DEFLATE: u16 = 0x0008;
    pub const JBIG_T85: u16 = 0x0009;
    pub const JBIG_T43: u16 = 0x000A;
    pub const NEXT: u16 = 0x7FFE;
    pub const PACKBITS: u16 = 0x8005;
    pub const THUNDERSCAN: u16 = 0x8029;
    pub const RASTERPADDING: u16 = 0x807F;
    pub const RLE_LINEWORK: u16 = 0x8080;
    pub const RLE_HIGH_RES: u16 = 0x8081;
    pub const RLE_BINARY_LINE: u16 = 0x8082;
    pub const DEFLATE_PKZIP: u16 = 0x80B2;
    pub const KODAK_DCS: u16 = 0x80B3;
    pub const JBIG: u16 = 0x8765;
    pub const JPEG2000: u16 = 0x8798;
    pub const NIKON_NEF: u16 = 0x8799;
    pub const JBIG2: u16 = 0x879B;
}

/// Photometic Interpretation magic
pub mod photometic_interpretation {
    pub const WHITEISZERO: u16 = 0x0000;
    pub const BLACKISZERO: u16 = 0x0001;
    pub const RGB: u16 = 0x0002;
    pub const RGB_PALETTE: u16 = 0x0003;
    pub const TRANSPARENCY_MASK: u16 = 0x0004;
    pub const CMYK: u16 = 0x0005;
    pub const YCBCR: u16 = 0x0006;
    pub const CIELAB: u16 = 0x0008;
}
