/// Header magic
pub mod header_magic {
    pub const LITTLE_ENDIAN_MAGIC: [u8; 2] = [b'I', b'I'];
    pub const BIG_ENDIAN_MAGIC: [u8; 2] = [b'M', b'M'];
    pub const VERSION_MAGIC: u16 = 42;
}

/// IFD Field types
pub mod ifd_field_type_magic {
    // Baseline
    pub const IFD_TYPE_BYTE: u16 = 0x0001;
    pub const IFD_TYPE_ASCII: u16 = 0x0002;
    pub const IFD_TYPE_SHORT: u16 = 0x0003;
    pub const IFD_TYPE_LONG: u16 = 0x0004;
    pub const IFD_TYPE_RATIONAL: u16 = 0x0005;

    // Extended
    pub const IFD_TYPE_SBYTE: u16 = 0x0006;
    pub const IFD_TYPE_UNDEFINED: u16 = 0x0007;
    pub const IFD_TYPE_SSHORT: u16 = 0x0008;
    pub const IFD_TYPE_SLONG: u16 = 0x0009;
    pub const IFD_TYPE_SRATIONAL: u16 = 0x000A;
    pub const IFD_TYPE_FLOAT: u16 = 0x000B;
    pub const IFD_TYPE_DOUBLE: u16 = 0x000C;
}
