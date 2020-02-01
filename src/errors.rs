use failure::Fail;

/// An error encountered during extraction of a field from from an IFD
#[derive(Fail, Debug, Clone, Copy)]
pub enum FieldExtractionError {
    #[fail(display = "Tag has wrong data type")]
    WrongDataType,
    #[fail(display = "Tag contains insufficient data")]
    InsufficientData,
    #[fail(display = "Missing tag {:X}", tag)]
    MissingTag {
        tag: u16,
    }
}

