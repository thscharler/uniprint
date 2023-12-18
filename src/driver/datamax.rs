use std::io::Write;

use crate::{Driver, Job, PrintJob};

const STX: char = '\x02';
const CR: char = '\x0D';

/// Datamax driver.
pub struct Datamax<J: Job> {
    pub print: J,
}

/// Constants for datamax.
pub enum Rotate {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

/// Constants for datamax.
pub enum FeedSpeed {
    Speed50mm,
    Speed76mm,
    Speed101mm,
    Speed127mm,
    Speed152mm,
    Speed177mm,
    Speed203mm,
    Speed228mm,
    Speed254mm,
    Speed279mm,
    Speed304mm,
}

/// Constants for datamax.
pub enum ScaleSize {
    S4,
    S6,
    S8,
    S10,
    S12,
    S14,
    S18,
    S24,
    S30,
    S36,
    S48,
    S72,
}

impl<J: Job> Driver for Datamax<J> {
    fn new<K: Job>(pr_name: &str, doc_name: &str) -> std::io::Result<Self> {
        Ok(Self {
            print: J::new(pr_name, doc_name)?,
        })
    }

    fn start_page(&mut self) -> std::io::Result<()> {
        self.print.start_page()
    }

    fn end_page(&mut self) -> std::io::Result<()> {
        self.print.end_page()
    }

    fn close(&mut self) -> std::io::Result<()> {
        self.print.close()
    }
}

impl<J: Job> Write for Datamax<J> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.print.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.print.flush()
    }
}

impl Datamax<PrintJob> {
    /// Opens a Spooler and starts a new Job.
    pub fn new_printjob(printer_name: &str, doc_name: &str) -> Result<Self, std::io::Error> {
        Ok(Self {
            print: PrintJob::new(printer_name, doc_name)?,
        })
    }
}

impl<J: Job> Datamax<J> {
    /// Label starten.
    pub fn start_label(&mut self) -> Result<(), std::io::Error> {
        write!(self.print, "{}", STX)?;
        write!(self.print, "L")?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }

    /// End label and print.
    pub fn end_label(&mut self) -> Result<(), std::io::Error> {
        write!(self.print, "E")?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }

    /// Density (0..30)
    pub fn print_density(&mut self, density: u8) -> Result<(), std::io::Error> {
        write!(self.print, "H{:02}", density)?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }

    /// Set to metric
    pub fn metric(&mut self) -> Result<(), std::io::Error> {
        write!(self.print, "m")?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }

    /// Speed for printing
    pub fn printable_speed(&mut self, speed: FeedSpeed) -> Result<(), std::io::Error> {
        write!(self.print, "P{}", feed_speed(speed))?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }

    /// Speed for empty spaces
    pub fn unprintable_speed(&mut self, speed: FeedSpeed) -> Result<(), std::io::Error> {
        write!(self.print, "S{}", feed_speed(speed))?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }

    /// Speed for backfeed
    pub fn backfeed_speed(&mut self, speed: FeedSpeed) -> Result<(), std::io::Error> {
        write!(self.print, "p{}", feed_speed(speed))?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }

    /// Pixel sizes. Horizontal (1,2) and vertical (1,2,3)
    pub fn pixel_size(&mut self, size_hor: u8, size_vert: u8) -> Result<(), std::io::Error> {
        write!(self.print, "D{:01}{:01}", size_hor, size_vert)?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }

    /// Copies to print
    pub fn copies(&mut self, copies: u16) -> Result<(), std::io::Error> {
        write!(self.print, "Q{:04}", copies)?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }

    /// Spacing between characters.
    pub fn spacing(&mut self, space: u8) -> Result<(), std::io::Error> {
        write!(self.print, "\x1bP{:02}", space)?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }

    /// Print reverse
    pub fn reverse(&mut self) -> Result<(), std::io::Error> {
        write!(self.print, "A5")?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }

    /// Print overlayed
    pub fn normal(&mut self) -> Result<(), std::io::Error> {
        write!(self.print, "A3")?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }

    /// Horizontal offset
    pub fn offset_x(&mut self, dist: u32) -> Result<(), std::io::Error> {
        write!(self.print, "C{:04}", dist)?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }

    /// Text output
    /// font (0..8)
    /// expand (1..24)
    /// row (0..9999)
    /// col (0.410)
    pub fn text_sys(
        &mut self,
        rotate: Rotate,
        font: u8,
        hor_expand: u8,
        vert_expand: u8,
        row_pos: u16,
        col_pos: u16,
        data: &str,
    ) -> Result<(), std::io::Error> {
        let data_enc = yore::code_pages::CP850.encode_lossy(data, b'_');

        write!(
            self.print,
            "{:1}",
            match rotate {
                Rotate::Rotate0 => 1,
                Rotate::Rotate90 => 2,
                Rotate::Rotate180 => 3,
                Rotate::Rotate270 => 4,
            }
        )?;
        write!(self.print, "{:1}", font)?;
        write!(self.print, "{:1}", expansion(hor_expand))?;
        write!(self.print, "{:1}", expansion(vert_expand))?;
        write!(self.print, "000")?;
        write!(self.print, "{:04}", row_pos)?;
        write!(self.print, "{:04}", col_pos)?;
        self.print.write(data_enc.as_ref())?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }

    /// Text output, scaling font.
    /// expand (1..24)
    /// size (4..72pt)
    /// row (0..9999)
    /// col (0.410)
    pub fn text_scale(
        &mut self,
        rotate: Rotate,
        hor_expand: u8,
        vert_expand: u8,
        bold: bool,
        size: ScaleSize,
        row_pos: u16,
        col_pos: u16,
        data: &str,
    ) -> Result<(), std::io::Error> {
        let data_enc = yore::code_pages::CP850.encode_lossy(data, b'_');

        write!(
            self.print,
            "{:1}",
            match rotate {
                Rotate::Rotate0 => 1,
                Rotate::Rotate90 => 2,
                Rotate::Rotate180 => 3,
                Rotate::Rotate270 => 4,
            }
        )?;
        write!(self.print, "{:1}", 9)?;
        write!(self.print, "{:1}", expansion(hor_expand))?;
        write!(self.print, "{:1}", expansion(vert_expand))?;
        let bold = if bold { "C" } else { "A" };
        write!(self.print, "{}{:02}", bold, scale_size(size))?;
        write!(self.print, "{:04}", row_pos)?;
        write!(self.print, "{:04}", col_pos)?;
        self.print.write(data_enc.as_ref())?;
        write!(self.print, "{}", CR)?;
        Ok(())
    }
}

fn feed_speed(speed: FeedSpeed) -> u8 {
    match speed {
        FeedSpeed::Speed50mm => b'1',
        FeedSpeed::Speed76mm => b'3',
        FeedSpeed::Speed101mm => b'4',
        FeedSpeed::Speed127mm => b'5',
        FeedSpeed::Speed152mm => b'6',
        FeedSpeed::Speed177mm => b'7',
        FeedSpeed::Speed203mm => b'8',
        FeedSpeed::Speed228mm => b'9',
        FeedSpeed::Speed254mm => b'a',
        FeedSpeed::Speed279mm => b'b',
        FeedSpeed::Speed304mm => b'c',
    }
}

fn scale_size(size: ScaleSize) -> u8 {
    match size {
        ScaleSize::S4 => 4,
        ScaleSize::S6 => 6,
        ScaleSize::S8 => 8,
        ScaleSize::S10 => 10,
        ScaleSize::S12 => 12,
        ScaleSize::S14 => 14,
        ScaleSize::S18 => 18,
        ScaleSize::S24 => 24,
        ScaleSize::S30 => 30,
        ScaleSize::S36 => 36,
        ScaleSize::S48 => 48,
        ScaleSize::S72 => 72,
    }
}

fn expansion(expand: u8) -> char {
    match expand {
        1..=9 => ('0' as u8 + expand) as char,
        10..=24 => ('@' as u8 + expand) as char,
        _ => '1',
    }
}
