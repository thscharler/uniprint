//! Tries to give a somewhat unified view over windows and cups printing.
//!
//! This is a impossible job, but listing printers and sending raw data
//! can be accomplished.
//!
//! For the rest: State and parameters are available, but system specific.
//!
use std::alloc::LayoutError;
use std::error::Error;
use std::ffi::NulError;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::num::ParseIntError;

#[cfg(target_os = "linux")]
pub use linux::{
    default_printer, list_printers, printer_attr, ColorMode, Duplex, Finishings, Format, Info,
    LinuxPrintJob as PrintJob, Orientation, PaperSize, PaperSource, PaperType, Quality,
};
#[cfg(target_os = "windows")]
pub use windows::{
    default_printer, list_printers, printer_attr, Collate, ColorMode, Duplex, Format, Info,
    Orientation, PaperSize, PaperSource, PaperType, Quality, TrueType, WindowsPrintJob as PrintJob,
};

/// Maps the system specific states to these basic flags.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Status {
    Idle,
    Busy,
    Stopped,
    Warn,
    Error,
}

pub mod driver;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

/// Trait for a printer driver.
///
/// Wraps around a Job. The impl provides higher level functions for
/// actually printing stuff.
pub trait Driver: Write {
    /// Create a new printjob.
    fn new(pr_name: &str, doc_name: &str) -> std::io::Result<Self>
    where
        Self: Sized;

    /// Create a new printjob.
    fn new_with(pr_name: &str, doc_name: &str, param: &JobParam) -> std::io::Result<Self>
    where
        Self: Sized;

    /// Start a new page. Hint to the printing system.
    fn start_page(&mut self) -> std::io::Result<()>;
    /// End a page. Hint to the printing system.
    fn end_page(&mut self) -> std::io::Result<()>;
    /// End the document.
    fn close(&mut self) -> std::io::Result<()>;
}

/// Job parameters.
///
/// Partly unifies the parameter names, and tries to sync the value-enums
/// so that the same values have the same name in both systems.
/// But it will provide all documented values for each system and an
/// additional escape hatch for undocumented values.
#[derive(Default, Clone, Debug)]
pub struct JobParam {
    /// Output format.
    pub data_format: Format,
    pub copies: Option<u32>,
    #[cfg(target_os = "linux")]
    pub finishings: Option<Finishings>,
    pub paper_size: Option<PaperSize>,
    pub paper_source: Option<PaperSource>,
    pub paper_type: Option<PaperType>,
    #[cfg(target_os = "linux")]
    pub number_up: Option<u32>,
    pub orientation: Option<Orientation>,
    pub color: Option<ColorMode>,
    pub quality: Option<Quality>,
    pub duplex: Option<Duplex>,
    /// Length in 1/10mm
    #[cfg(target_os = "windows")]
    pub paper_length: Option<i16>,
    /// Width in 1/10mm
    #[cfg(target_os = "windows")]
    pub paper_width: Option<i16>,
    /// Scale in scale/100
    #[cfg(target_os = "windows")]
    pub scale: Option<i16>,
    /// DPI
    #[cfg(target_os = "windows")]
    pub y_resolution: Option<i16>,
    #[cfg(target_os = "windows")]
    pub tt_option: Option<TrueType>,
    #[cfg(target_os = "windows")]
    pub collate: Option<Collate>,
}

/// Printer errors.
#[derive(Debug, Clone)]
pub enum PrintError {
    /// Error from the printing system.
    Print(String),
    /// Printer not found.
    NotFound,
    /// No default printer.
    NoDefaultPrinter,
    /// Already working on a document.
    DocumentOpen,
    /// C string conversion error.
    InteriorNulInCStr,
    /// Memory layout error.
    LayoutError,
    /// ParseIntError
    ParseIntError,
}

impl Error for PrintError {}

impl Display for PrintError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            PrintError::Print(v) => write!(f, "{}", v),
            PrintError::NotFound => write!(f, "Printer not found."),
            PrintError::NoDefaultPrinter => write!(f, "No default printer."),
            PrintError::InteriorNulInCStr => write!(f, "Invalid NUL found."),
            PrintError::ParseIntError => write!(f, "Parse int error."),
            PrintError::DocumentOpen => write!(f, "Document already open."),
            PrintError::LayoutError => write!(f, "Can't create memory layout."),
        }
    }
}

impl From<NulError> for PrintError {
    fn from(_: NulError) -> Self {
        PrintError::InteriorNulInCStr
    }
}

impl From<LayoutError> for PrintError {
    fn from(_: LayoutError) -> Self {
        PrintError::LayoutError
    }
}

impl From<ParseIntError> for PrintError {
    fn from(_: ParseIntError) -> Self {
        PrintError::ParseIntError
    }
}
