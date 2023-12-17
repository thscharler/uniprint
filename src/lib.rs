use std::error::Error;
use std::ffi::NulError;
use std::fmt::{Display, Formatter};
use std::io::Write;

#[cfg(target_os = "linux")]
pub use linux::{default_printer, list_printers, printer_attr};
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
pub trait Driver {
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

/// Abstracts the PrintJob.
pub trait Job: Write {
    /// Create a new printjob.
    fn new(pr_name: &str, doc_name: &str) -> std::io::Result<Self>
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
    /// No default printer.
    NoDefaultPrinter,
    /// C string conversion error.
    InteriorNulInCStr,
    /// Printer not found.
    NotFound,
}

impl Error for PrintError {}

impl Display for PrintError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            PrintError::Print(v) => write!(f, "{}", v),
            PrintError::NoDefaultPrinter => write!(f, "No default printer."),
            PrintError::InteriorNulInCStr => write!(f, "Invalid NUL found."),
            PrintError::NotFound => write!(f, "Printer not found."),
        }
    }
}

impl From<NulError> for PrintError {
    fn from(_: NulError) -> Self {
        PrintError::InteriorNulInCStr
    }
}
