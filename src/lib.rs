use std::error::Error;
use std::ffi::NulError;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::num::ParseIntError;

#[cfg(target_os = "linux")]
pub use linux::{
    default_printer, list_printers, printer_attr, ColorMode, Duplex, Finishings, Info, Orientation,
    PaperSize, PaperSource, PaperType, Quality, Format 
};
#[cfg(target_os = "windows")]
pub use windows::{default_printer, list_printers, printer_attr, Info};

pub enum Status {
    /// CUPS: printer-state=3
    /// WIN: no status bit set
    Idle,
    /// CUPS: printer-state=4
    /// WIN: PRINTER_STATUS_BUSY
    Busy,
    /// CUPS: printer-state=5
    /// WIN: PRINTER_STATUS_OFFLINE, PRINTER_STATUS_NOT_AVAILABLE
    Stopped,
    /// CUPS: printer-state=5
    ///     printer-state-reasons: ...
    /// WIN: ...
    Warn,
    /// CUPS: printer-state=5
    ///     printer-state-reasons: ...
    /// WIN: ...
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
    /// Output the driver results to this write.
    fn new<J: Job>(pr_name: &str, doc_name: &str) -> std::io::Result<Self>
    where
        Self: Sized;
    /// Start a new page.
    fn start_page(&mut self) -> std::io::Result<()>;
    /// End a page.
    fn end_page(&mut self) -> std::io::Result<()>;
    /// End the document.
    fn close(&mut self) -> std::io::Result<()>;
}

#[derive(Default, Clone, Debug)]
pub struct JobParam {
    pub data_format: Format, 
    pub copies: Option<u32>,
    pub finishings: Option<Finishings>,
    pub paper_size: Option<PaperSize>,
    pub paper_source: Option<PaperSource>,
    pub paper_type: Option<PaperType>,
    pub number_up: Option<u32>,
    pub orientation: Option<Orientation>,
    pub color: Option<ColorMode>,
    pub quality: Option<Quality>,
    pub duplex: Option<Duplex>,
}

/// Abstracts the PrintJob.
pub trait Job: Write {
    /// Create a new printjob.
    fn new(pr_name: &str, doc_name: &str) -> std::io::Result<Self>
    where
        Self: Sized;

    /// Create a new printjob.
    fn new_with(pr_name: &str, doc_name: &str, param: &JobParam) -> std::io::Result<Self>
    where
        Self: Sized;

    /// Informs the print-system of a new page. Emits no bytes to the actual printer.
    fn start_page(&self) -> std::io::Result<()>;
    /// Informs the print-system of a page end. Emits no bytes to the actual printer.
    fn end_page(&self) -> std::io::Result<()>;
    /// Closes and sends of the printjob.
    fn close(&mut self) -> std::io::Result<()>;
}

/// A print job.
pub struct PrintJob {
    #[cfg(target_os = "windows")]
    job: windows::WindowsPrintJob,

    #[cfg(target_os = "linux")]
    job: linux::LinuxPrintJob,
}

impl Write for PrintJob {
    /// Write bytes to a printer.
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.job.write(buf)
    }

    /// ...
    fn flush(&mut self) -> std::io::Result<()> {
        self.job.flush()
    }
}

impl Job for PrintJob {
    /// Create a new printjob.
    fn new(pr_name: &str, doc_name: &str) -> std::io::Result<Self> {
        Ok(Self {
            #[cfg(target_os = "windows")]
            job: windows::WindowsPrintJob::new(pr_name, doc_name)?,
            #[cfg(target_os = "linux")]
            job: linux::LinuxPrintJob::new(pr_name, doc_name)?,
        })
    }

    fn new_with(pr_name: &str, doc_name: &str, param: &JobParam) -> std::io::Result<Self> {
        Ok(Self {
            #[cfg(target_os = "windows")]
            job: windows::WindowsPrintJob::new_with(pr_name, doc_name, param)?,
            #[cfg(target_os = "linux")]
            job: linux::LinuxPrintJob::new_with(pr_name, doc_name, param)?,
        })
    }

    /// Informs the print-system of a new page. Emits no bytes to the actual printer.
    fn start_page(&self) -> std::io::Result<()> {
        self.job.start_page()
    }

    /// Informs the print-system of a page end. Emits no bytes to the actual printer.
    fn end_page(&self) -> std::io::Result<()> {
        self.job.end_page()
    }

    /// Closes and sends of the printjob.
    fn close(&mut self) -> std::io::Result<()> {
        self.job.close()
    }
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
    /// C string conversion error.
    InteriorNulInCStr,
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
        }
    }
}

impl From<NulError> for PrintError {
    fn from(_: NulError) -> Self {
        PrintError::InteriorNulInCStr
    }
}

impl From<ParseIntError> for PrintError {
    fn from(_: ParseIntError) -> Self {
        PrintError::ParseIntError
    }
}
