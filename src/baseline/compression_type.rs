use crate::metadata::constants::compression_type_magic::*;

/// Defines a Decoding Type for use between decoders.
#[derive(Debug, Clone, Copy)]
pub enum CompressionType {
    Uncompressed,
    CCITT1D,
    Group3Fax,
    Group4Fax,
    LZW,
    JPEG,
    NewJPEG,
    AdobeDEFLATE,
    JBigT85,
    JBigT43,
    NeXT,
    PackBits,
    Thunderscan,
    Rasterpadding,
    LineworkRLE,
    HighResRLE,
    BinaryLineRLE,
    DEFLATEPKZIP,
    KodakDCS,
    JBIG,
    JPEG2000,
    NikonNEF,
    JBIG2,
}

impl Into<u16> for CompressionType {
    fn into(self) -> u16 {
        match self {
            Self::Uncompressed => COMPRESSION_TYPE_UNCOMPRESSED,
            Self::CCITT1D => COMPRESSION_TYPE_CCITT_1D,
            Self::Group3Fax => COMPRESSION_TYPE_GROUP_3_FAX,
            Self::Group4Fax => COMPRESSION_TYPE_GROUP_4_FAX,
            Self::LZW => COMPRESSION_TYPE_LZW,
            Self::JPEG => COMPRESSION_TYPE_JPEG,
            Self::NewJPEG => COMPRESSION_TYPE_NEW_JPEG,
            Self::AdobeDEFLATE => COMPRESSION_TYPE_ADOBE_DEFLATE,
            Self::JBigT85 => COMPRESSION_TYPE_JBIG_T85,
            Self::JBigT43 => COMPRESSION_TYPE_JBIG_T43,
            Self::NeXT => COMPRESSION_TYPE_NEXT,
            Self::PackBits => COMPRESSION_TYPE_PACKBITS,
            Self::Thunderscan => COMPRESSION_TYPE_THUNDERSCAN,
            Self::Rasterpadding => COMPRESSION_TYPE_RASTERPADDING,
            Self::LineworkRLE => COMPRESSION_TYPE_RLE_LINEWORK,
            Self::HighResRLE => COMPRESSION_TYPE_RLE_HIGH_RES,
            Self::BinaryLineRLE => COMPRESSION_TYPE_RLE_BINARY_LINE,
            Self::DEFLATEPKZIP => COMPRESSION_TYPE_DEFLATE_PKZIP,
            Self::KodakDCS => COMPRESSION_TYPE_KODAK_DCS,
            Self::JBIG => COMPRESSION_TYPE_JBIG,
            Self::JPEG2000 => COMPRESSION_TYPE_JPEG2000,
            Self::NikonNEF => COMPRESSION_TYPE_NIKON_NEF,
            Self::JBIG2 => COMPRESSION_TYPE_JBIG2,
        }
    }
}

impl CompressionType {
    /// Convert an integer into the specified CompressionType. Returns None if unrecognized.
    pub fn from_iteger(integer: u16) -> Option<Self> {
        match integer {
            COMPRESSION_TYPE_UNCOMPRESSED => Some(Self::Uncompressed),
            COMPRESSION_TYPE_CCITT_1D => Some(Self::CCITT1D),
            COMPRESSION_TYPE_GROUP_3_FAX => Some(Self::Group3Fax),
            COMPRESSION_TYPE_GROUP_4_FAX => Some(Self::Group4Fax),
            COMPRESSION_TYPE_LZW => Some(Self::LZW),
            COMPRESSION_TYPE_JPEG => Some(Self::JPEG),
            COMPRESSION_TYPE_NEW_JPEG => Some(Self::NewJPEG),
            COMPRESSION_TYPE_ADOBE_DEFLATE => Some(Self::AdobeDEFLATE),
            COMPRESSION_TYPE_JBIG_T85 => Some(Self::JBigT85),
            COMPRESSION_TYPE_JBIG_T43 => Some(Self::JBigT43),
            COMPRESSION_TYPE_NEXT => Some(Self::NeXT),
            COMPRESSION_TYPE_PACKBITS => Some(Self::PackBits),
            COMPRESSION_TYPE_THUNDERSCAN => Some(Self::Thunderscan),
            COMPRESSION_TYPE_RASTERPADDING => Some(Self::Rasterpadding),
            COMPRESSION_TYPE_RLE_LINEWORK => Some(Self::LineworkRLE),
            COMPRESSION_TYPE_RLE_HIGH_RES => Some(Self::HighResRLE),
            COMPRESSION_TYPE_RLE_BINARY_LINE => Some(Self::BinaryLineRLE),
            COMPRESSION_TYPE_DEFLATE_PKZIP => Some(Self::DEFLATEPKZIP),
            COMPRESSION_TYPE_KODAK_DCS => Some(Self::KodakDCS),
            COMPRESSION_TYPE_JBIG => Some(Self::JBIG),
            COMPRESSION_TYPE_JPEG2000 => Some(Self::JPEG2000),
            COMPRESSION_TYPE_NIKON_NEF => Some(Self::NikonNEF),
            COMPRESSION_TYPE_JBIG2 => Some(Self::JBIG2),
            _ => None,
        }
    }
}
