use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ErrorPolicy {
    Fail,
    Skip,
}

impl ErrorPolicy {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Fail => "fail",
            Self::Skip => "skip",
        }
    }
}

impl FromStr for ErrorPolicy {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "fail" => Ok(Self::Fail),
            "skip" => Ok(Self::Skip),
            _ => Err(format!("unknown error policy: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FileProcessingError {
    pub(super) path: PathBuf,
    pub(super) stage: &'static str,
    pub(super) message: String,
}
