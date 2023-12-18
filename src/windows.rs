use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::cmp::min;
use std::ffi::{c_void, OsString};
use std::io::{ErrorKind, Write};
use std::iter::once;
use std::mem::{align_of, size_of};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr;
use std::ptr::slice_from_raw_parts;

use libc::wcslen;
use windows_sys::core::{PCWSTR, PWSTR};
use windows_sys::Win32::Foundation::{GetLastError, LocalFree, FALSE, HANDLE, HLOCAL, TRUE};
use windows_sys::Win32::Graphics::Gdi::{
    CCHFORMNAME, DEVMODEW, DEVMODEW_0, DEVMODEW_0_0, DEVMODEW_1, DMBIN_AUTO, DMBIN_CASSETTE,
    DMBIN_ENVELOPE, DMBIN_ENVMANUAL, DMBIN_FORMSOURCE, DMBIN_LARGECAPACITY, DMBIN_LARGEFMT,
    DMBIN_LOWER, DMBIN_MANUAL, DMBIN_MIDDLE, DMBIN_SMALLFMT, DMBIN_TRACTOR, DMBIN_UPPER,
    DMBIN_USER, DMCOLLATE_FALSE, DMCOLLATE_TRUE, DMCOLOR_COLOR, DMCOLOR_MONOCHROME,
    DMDUP_HORIZONTAL, DMDUP_SIMPLEX, DMDUP_VERTICAL, DMMEDIA_GLOSSY, DMMEDIA_STANDARD,
    DMMEDIA_TRANSPARENCY, DMMEDIA_USER, DMORIENT_LANDSCAPE, DMORIENT_PORTRAIT, DMPAPER_10X11,
    DMPAPER_10X14, DMPAPER_11X17, DMPAPER_12X11, DMPAPER_15X11, DMPAPER_9X11, DMPAPER_A2,
    DMPAPER_A3, DMPAPER_A3_EXTRA, DMPAPER_A3_EXTRA_TRANSVERSE, DMPAPER_A3_ROTATED,
    DMPAPER_A3_TRANSVERSE, DMPAPER_A4, DMPAPER_A4SMALL, DMPAPER_A4_EXTRA, DMPAPER_A4_PLUS,
    DMPAPER_A4_ROTATED, DMPAPER_A4_TRANSVERSE, DMPAPER_A5, DMPAPER_A5_EXTRA, DMPAPER_A5_ROTATED,
    DMPAPER_A5_TRANSVERSE, DMPAPER_A6, DMPAPER_A6_ROTATED, DMPAPER_A_PLUS, DMPAPER_B4,
    DMPAPER_B4_JIS_ROTATED, DMPAPER_B5, DMPAPER_B5_EXTRA, DMPAPER_B5_JIS_ROTATED,
    DMPAPER_B5_TRANSVERSE, DMPAPER_B6_JIS, DMPAPER_B6_JIS_ROTATED, DMPAPER_B_PLUS, DMPAPER_CSHEET,
    DMPAPER_DBL_JAPANESE_POSTCARD, DMPAPER_DBL_JAPANESE_POSTCARD_ROTATED, DMPAPER_DSHEET,
    DMPAPER_ENV_10, DMPAPER_ENV_11, DMPAPER_ENV_12, DMPAPER_ENV_14, DMPAPER_ENV_9, DMPAPER_ENV_B4,
    DMPAPER_ENV_B5, DMPAPER_ENV_B6, DMPAPER_ENV_C3, DMPAPER_ENV_C4, DMPAPER_ENV_C5, DMPAPER_ENV_C6,
    DMPAPER_ENV_C65, DMPAPER_ENV_DL, DMPAPER_ENV_INVITE, DMPAPER_ENV_ITALY, DMPAPER_ENV_MONARCH,
    DMPAPER_ENV_PERSONAL, DMPAPER_ESHEET, DMPAPER_EXECUTIVE, DMPAPER_FANFOLD_LGL_GERMAN,
    DMPAPER_FANFOLD_STD_GERMAN, DMPAPER_FANFOLD_US, DMPAPER_FOLIO, DMPAPER_ISO_B4,
    DMPAPER_JAPANESE_POSTCARD, DMPAPER_JAPANESE_POSTCARD_ROTATED, DMPAPER_JENV_CHOU3,
    DMPAPER_JENV_CHOU3_ROTATED, DMPAPER_JENV_CHOU4, DMPAPER_JENV_CHOU4_ROTATED, DMPAPER_JENV_KAKU2,
    DMPAPER_JENV_KAKU2_ROTATED, DMPAPER_JENV_KAKU3, DMPAPER_JENV_KAKU3_ROTATED, DMPAPER_JENV_YOU4,
    DMPAPER_JENV_YOU4_ROTATED, DMPAPER_LEDGER, DMPAPER_LEGAL, DMPAPER_LEGAL_EXTRA, DMPAPER_LETTER,
    DMPAPER_LETTERSMALL, DMPAPER_LETTER_EXTRA, DMPAPER_LETTER_EXTRA_TRANSVERSE,
    DMPAPER_LETTER_PLUS, DMPAPER_LETTER_ROTATED, DMPAPER_LETTER_TRANSVERSE, DMPAPER_NOTE,
    DMPAPER_P16K, DMPAPER_P16K_ROTATED, DMPAPER_P32K, DMPAPER_P32KBIG, DMPAPER_P32KBIG_ROTATED,
    DMPAPER_P32K_ROTATED, DMPAPER_PENV_1, DMPAPER_PENV_10, DMPAPER_PENV_10_ROTATED,
    DMPAPER_PENV_1_ROTATED, DMPAPER_PENV_2, DMPAPER_PENV_2_ROTATED, DMPAPER_PENV_3,
    DMPAPER_PENV_3_ROTATED, DMPAPER_PENV_4, DMPAPER_PENV_4_ROTATED, DMPAPER_PENV_5,
    DMPAPER_PENV_5_ROTATED, DMPAPER_PENV_6, DMPAPER_PENV_6_ROTATED, DMPAPER_PENV_7,
    DMPAPER_PENV_7_ROTATED, DMPAPER_PENV_8, DMPAPER_PENV_8_ROTATED, DMPAPER_PENV_9,
    DMPAPER_PENV_9_ROTATED, DMPAPER_QUARTO, DMPAPER_RESERVED_48, DMPAPER_RESERVED_49,
    DMPAPER_STATEMENT, DMPAPER_TABLOID, DMPAPER_TABLOID_EXTRA, DMPAPER_USER, DMRES_DRAFT,
    DMRES_HIGH, DMRES_LOW, DMRES_MEDIUM, DMTT_BITMAP, DMTT_DOWNLOAD, DMTT_DOWNLOAD_OUTLINE,
    DMTT_SUBDEV, DM_COLLATE, DM_COLOR, DM_COPIES, DM_DEFAULTSOURCE, DM_DITHERTYPE, DM_DUPLEX,
    DM_FORMNAME, DM_ICMINTENT, DM_ICMMETHOD, DM_MEDIATYPE, DM_NUP, DM_ORIENTATION, DM_PAPERLENGTH,
    DM_PAPERSIZE, DM_PAPERWIDTH, DM_PRINTQUALITY, DM_SCALE, DM_SPECVERSION, DM_TTOPTION,
    DM_YRESOLUTION,
};
use windows_sys::Win32::Graphics::Printing::{
    ClosePrinter, EndDocPrinter, EndPagePrinter, EnumPrintersW, GetDefaultPrinterW, GetPrinterW,
    OpenPrinterW, StartDocPrinterW, StartPagePrinter, WritePrinter, DOC_INFO_1W,
    PRINTER_ACCESS_USE, PRINTER_ATTRIBUTE_DEFAULT, PRINTER_ATTRIBUTE_DIRECT,
    PRINTER_ATTRIBUTE_DO_COMPLETE_FIRST, PRINTER_ATTRIBUTE_ENABLE_BIDI,
    PRINTER_ATTRIBUTE_ENABLE_DEVQ, PRINTER_ATTRIBUTE_ENTERPRISE_CLOUD, PRINTER_ATTRIBUTE_FAX,
    PRINTER_ATTRIBUTE_FRIENDLY_NAME, PRINTER_ATTRIBUTE_HIDDEN, PRINTER_ATTRIBUTE_KEEPPRINTEDJOBS,
    PRINTER_ATTRIBUTE_LOCAL, PRINTER_ATTRIBUTE_MACHINE, PRINTER_ATTRIBUTE_NETWORK,
    PRINTER_ATTRIBUTE_PER_USER, PRINTER_ATTRIBUTE_PUBLISHED, PRINTER_ATTRIBUTE_PUSHED_MACHINE,
    PRINTER_ATTRIBUTE_PUSHED_USER, PRINTER_ATTRIBUTE_QUEUED, PRINTER_ATTRIBUTE_RAW_ONLY,
    PRINTER_ATTRIBUTE_SHARED, PRINTER_ATTRIBUTE_TS, PRINTER_ATTRIBUTE_TS_GENERIC_DRIVER,
    PRINTER_ATTRIBUTE_WORK_OFFLINE, PRINTER_DEFAULTSW, PRINTER_ENUM_LOCAL, PRINTER_INFO_2W,
    PRINTER_INFO_4W, PRINTER_STATUS_BUSY, PRINTER_STATUS_DOOR_OPEN, PRINTER_STATUS_ERROR,
    PRINTER_STATUS_INITIALIZING, PRINTER_STATUS_IO_ACTIVE, PRINTER_STATUS_MANUAL_FEED,
    PRINTER_STATUS_NOT_AVAILABLE, PRINTER_STATUS_NO_TONER, PRINTER_STATUS_OFFLINE,
    PRINTER_STATUS_OUTPUT_BIN_FULL, PRINTER_STATUS_OUT_OF_MEMORY, PRINTER_STATUS_PAGE_PUNT,
    PRINTER_STATUS_PAPER_JAM, PRINTER_STATUS_PAPER_OUT, PRINTER_STATUS_PAPER_PROBLEM,
    PRINTER_STATUS_PAUSED, PRINTER_STATUS_PENDING_DELETION, PRINTER_STATUS_POWER_SAVE,
    PRINTER_STATUS_PRINTING, PRINTER_STATUS_PROCESSING, PRINTER_STATUS_SERVER_UNKNOWN,
    PRINTER_STATUS_TONER_LOW, PRINTER_STATUS_USER_INTERVENTION, PRINTER_STATUS_WAITING,
    PRINTER_STATUS_WARMING_UP,
};
use windows_sys::Win32::System::Diagnostics::Debug::{
    FormatMessageW, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM,
    FORMAT_MESSAGE_IGNORE_INSERTS,
};

use crate::{JobParam, PrintError, Status};

#[derive(Default, Debug, Clone, Copy)]
pub enum Format {
    #[default]
    Raw,
    RawFFAppended,
    RawFFAuto,
    NtEmf1_003,
    NtEmf1_006,
    NtEmf1_007,
    NtEmf1_008,
    Text,
    XpsPass,
    Xps2Gdi,
}

#[repr(u32)]
#[derive(Default, Debug, Clone, Copy)]
pub enum PaperSize {
    Numeric(i16),
    #[default]
    Letter = DMPAPER_LETTER,
    Lettersmall = DMPAPER_LETTERSMALL,
    Tabloid = DMPAPER_TABLOID,
    Ledger = DMPAPER_LEDGER,
    Legal = DMPAPER_LEGAL,
    Statement = DMPAPER_STATEMENT,
    Executive = DMPAPER_EXECUTIVE,
    A3 = DMPAPER_A3,
    A4 = DMPAPER_A4,
    A4small = DMPAPER_A4SMALL,
    A5 = DMPAPER_A5,
    B4 = DMPAPER_B4,
    B5 = DMPAPER_B5,
    Folio = DMPAPER_FOLIO,
    Quarto = DMPAPER_QUARTO,
    Size10x14 = DMPAPER_10X14,
    Size11x17 = DMPAPER_11X17,
    Note = DMPAPER_NOTE,
    Env9 = DMPAPER_ENV_9,
    Env10 = DMPAPER_ENV_10,
    Env11 = DMPAPER_ENV_11,
    Env12 = DMPAPER_ENV_12,
    Env14 = DMPAPER_ENV_14,
    Csheet = DMPAPER_CSHEET,
    Dsheet = DMPAPER_DSHEET,
    Esheet = DMPAPER_ESHEET,
    EnvDl = DMPAPER_ENV_DL,
    EnvC5 = DMPAPER_ENV_C5,
    EnvC3 = DMPAPER_ENV_C3,
    EnvC4 = DMPAPER_ENV_C4,
    EnvC6 = DMPAPER_ENV_C6,
    EnvC65 = DMPAPER_ENV_C65,
    EnvB4 = DMPAPER_ENV_B4,
    EnvB5 = DMPAPER_ENV_B5,
    EnvB6 = DMPAPER_ENV_B6,
    EnvItaly = DMPAPER_ENV_ITALY,
    EnvMonarch = DMPAPER_ENV_MONARCH,
    EnvPersonal = DMPAPER_ENV_PERSONAL,
    FanfoldUs = DMPAPER_FANFOLD_US,
    FanfoldStdGerman = DMPAPER_FANFOLD_STD_GERMAN,
    FanfoldLglGerman = DMPAPER_FANFOLD_LGL_GERMAN,
    IsoB4 = DMPAPER_ISO_B4,
    JapanesePostcard = DMPAPER_JAPANESE_POSTCARD,
    Size9x11 = DMPAPER_9X11,
    Size10x11 = DMPAPER_10X11,
    Size15x11 = DMPAPER_15X11,
    EnvInvite = DMPAPER_ENV_INVITE,
    Reserved48 = DMPAPER_RESERVED_48,
    Reserved49 = DMPAPER_RESERVED_49,
    LetterExtra = DMPAPER_LETTER_EXTRA,
    LegalExtra = DMPAPER_LEGAL_EXTRA,
    TabloidExtra = DMPAPER_TABLOID_EXTRA,
    A4Extra = DMPAPER_A4_EXTRA,
    LetterTransverse = DMPAPER_LETTER_TRANSVERSE,
    A4Transverse = DMPAPER_A4_TRANSVERSE,
    LetterExtraTransverse = DMPAPER_LETTER_EXTRA_TRANSVERSE,
    APlus = DMPAPER_A_PLUS,
    BPlus = DMPAPER_B_PLUS,
    LetterPlus = DMPAPER_LETTER_PLUS,
    A4Plus = DMPAPER_A4_PLUS,
    A5Transverse = DMPAPER_A5_TRANSVERSE,
    B5Transverse = DMPAPER_B5_TRANSVERSE,
    A3Extra = DMPAPER_A3_EXTRA,
    A5Extra = DMPAPER_A5_EXTRA,
    B5Extra = DMPAPER_B5_EXTRA,
    A2 = DMPAPER_A2,
    A3Transverse = DMPAPER_A3_TRANSVERSE,
    A3ExtraTransverse = DMPAPER_A3_EXTRA_TRANSVERSE,
    DblJapanesePostcard = DMPAPER_DBL_JAPANESE_POSTCARD,
    A6 = DMPAPER_A6,
    JenvKaku2 = DMPAPER_JENV_KAKU2,
    JenvKaku3 = DMPAPER_JENV_KAKU3,
    JenvChou3 = DMPAPER_JENV_CHOU3,
    JenvChou4 = DMPAPER_JENV_CHOU4,
    LetterRotated = DMPAPER_LETTER_ROTATED,
    A3Rotated = DMPAPER_A3_ROTATED,
    A4Rotated = DMPAPER_A4_ROTATED,
    A5Rotated = DMPAPER_A5_ROTATED,
    B4JisRotated = DMPAPER_B4_JIS_ROTATED,
    B5JisRotated = DMPAPER_B5_JIS_ROTATED,
    JapanesePostcardRotated = DMPAPER_JAPANESE_POSTCARD_ROTATED,
    DblJapanesePostcardRotated = DMPAPER_DBL_JAPANESE_POSTCARD_ROTATED,
    A6Rotated = DMPAPER_A6_ROTATED,
    JenvKaku2Rotated = DMPAPER_JENV_KAKU2_ROTATED,
    JenvKaku3Rotated = DMPAPER_JENV_KAKU3_ROTATED,
    JenvChou3Rotated = DMPAPER_JENV_CHOU3_ROTATED,
    JenvChou4Rotated = DMPAPER_JENV_CHOU4_ROTATED,
    B6Jis = DMPAPER_B6_JIS,
    B6JisRotated = DMPAPER_B6_JIS_ROTATED,
    Size12x11 = DMPAPER_12X11,
    JenvYou4 = DMPAPER_JENV_YOU4,
    JenvYou4Rotated = DMPAPER_JENV_YOU4_ROTATED,
    P16k = DMPAPER_P16K,
    P32k = DMPAPER_P32K,
    P32kbig = DMPAPER_P32KBIG,
    Penv1 = DMPAPER_PENV_1,
    Penv2 = DMPAPER_PENV_2,
    Penv3 = DMPAPER_PENV_3,
    Penv4 = DMPAPER_PENV_4,
    Penv5 = DMPAPER_PENV_5,
    Penv6 = DMPAPER_PENV_6,
    Penv7 = DMPAPER_PENV_7,
    Penv8 = DMPAPER_PENV_8,
    Penv9 = DMPAPER_PENV_9,
    Penv10 = DMPAPER_PENV_10,
    P16kRotated = DMPAPER_P16K_ROTATED,
    P32kRotated = DMPAPER_P32K_ROTATED,
    P32kbigRotated = DMPAPER_P32KBIG_ROTATED,
    Penv1Rotated = DMPAPER_PENV_1_ROTATED,
    Penv2Rotated = DMPAPER_PENV_2_ROTATED,
    Penv3Rotated = DMPAPER_PENV_3_ROTATED,
    Penv4Rotated = DMPAPER_PENV_4_ROTATED,
    Penv5Rotated = DMPAPER_PENV_5_ROTATED,
    Penv6Rotated = DMPAPER_PENV_6_ROTATED,
    Penv7Rotated = DMPAPER_PENV_7_ROTATED,
    Penv8Rotated = DMPAPER_PENV_8_ROTATED,
    Penv9Rotated = DMPAPER_PENV_9_ROTATED,
    Penv10Rotated = DMPAPER_PENV_10_ROTATED,
    User = DMPAPER_USER,
}

impl PaperSize {
    fn discriminant(&self) -> u32 {
        // SAFETY: Because `Self` is marked `repr(u32)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u32` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *(self as *const PaperSize as *const u32) }
    }
}

#[repr(u32)]
#[derive(Default, Debug, Clone, Copy)]
pub enum PaperSource {
    Numeric(i16),
    Upper = DMBIN_UPPER,
    Lower = DMBIN_LOWER,
    Middle = DMBIN_MIDDLE,
    Manual = DMBIN_MANUAL,
    Envelope = DMBIN_ENVELOPE,
    Envmanual = DMBIN_ENVMANUAL,
    #[default]
    Auto = DMBIN_AUTO,
    Tractor = DMBIN_TRACTOR,
    Smallfmt = DMBIN_SMALLFMT,
    Largefmt = DMBIN_LARGEFMT,
    Largecapacity = DMBIN_LARGECAPACITY,
    Cassette = DMBIN_CASSETTE,
    Formsource = DMBIN_FORMSOURCE,
    User = DMBIN_USER,
}

impl PaperSource {
    fn discriminant(&self) -> u32 {
        // SAFETY: Because `Self` is marked `repr(u32)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u32` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *(self as *const PaperSource as *const u32) }
    }
}

#[repr(u32)]
#[derive(Default, Debug, Clone, Copy)]
pub enum PaperType {
    Numeric(u32),
    Glossy = DMMEDIA_GLOSSY,
    #[default]
    Standard = DMMEDIA_STANDARD,
    Transparency = DMMEDIA_TRANSPARENCY,
    User = DMMEDIA_USER,
}

impl PaperType {
    fn discriminant(&self) -> u32 {
        // SAFETY: Because `Self` is marked `repr(u32)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u32` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *(self as *const PaperType as *const u32) }
    }
}

#[repr(u32)]
#[derive(Default, Debug, Clone, Copy)]
pub enum Orientation {
    Numeric(i16),
    #[default]
    Portrait = DMORIENT_PORTRAIT,
    Landscape = DMORIENT_LANDSCAPE,
}

impl Orientation {
    fn discriminant(&self) -> u32 {
        // SAFETY: Because `Self` is marked `repr(u32)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u32` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *(self as *const Orientation as *const u32) }
    }
}

#[repr(i16)]
#[derive(Default, Debug, Clone, Copy)]
pub enum ColorMode {
    Numeric(i16),
    #[default]
    Monochrome = DMCOLOR_MONOCHROME,
    Color = DMCOLOR_COLOR,
}

impl ColorMode {
    fn discriminant(&self) -> i16 {
        // SAFETY: Because `Self` is marked `repr(i16)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u32` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *(self as *const ColorMode as *const i16) }
    }
}

#[repr(i32)]
#[derive(Default, Debug, Clone, Copy)]
pub enum Quality {
    Numeric(i16),
    Draft = DMRES_DRAFT,
    Low = DMRES_LOW,
    #[default]
    Normal = DMRES_MEDIUM,
    High = DMRES_HIGH,
}

impl Quality {
    fn discriminant(&self) -> i32 {
        // SAFETY: Because `Self` is marked `repr(i32)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `i32` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *(self as *const Quality as *const i32) }
    }
}

#[repr(i16)]
#[derive(Default, Debug, Clone, Copy)]
pub enum Duplex {
    Numeric(i16),
    #[default]
    Simplex = DMDUP_SIMPLEX,
    TwoSidedPortrait = DMDUP_VERTICAL,
    TwoSidedLandscape = DMDUP_HORIZONTAL,
}

impl Duplex {
    fn discriminant(&self) -> i16 {
        // SAFETY: Because `Self` is marked `repr(i16)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `i16` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *(self as *const Duplex as *const i16) }
    }
}

#[repr(i16)]
#[derive(Default, Debug, Clone, Copy)]
pub enum TrueType {
    Numeric(i16),
    Bitmap = DMTT_BITMAP,
    Download = DMTT_DOWNLOAD,
    DownloadOutline = DMTT_DOWNLOAD_OUTLINE,
    #[default]
    SubDev = DMTT_SUBDEV,
}

impl TrueType {
    fn discriminant(&self) -> i16 {
        // SAFETY: Because `Self` is marked `repr(i16)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `i16` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *(self as *const TrueType as *const i16) }
    }
}

#[repr(i16)]
#[derive(Default, Debug, Clone, Copy)]
pub enum Collate {
    Numeric(i16) = -1,
    #[default]
    CollateFalse = DMCOLLATE_FALSE,
    CollateTrue = DMCOLLATE_TRUE,
}

impl Collate {
    fn discriminant(&self) -> i16 {
        // SAFETY: Because `Self` is marked `repr(i16)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `i16` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *(self as *const Collate as *const i16) }
    }
}

impl PrintError {
    #[allow(dead_code)]
    pub(crate) fn io_error(e: PrintError) -> std::io::Error {
        std::io::Error::new(ErrorKind::Other, e)
    }

    pub(crate) fn last_io_error() -> std::io::Error {
        std::io::Error::new(ErrorKind::Other, PrintError::last_error())
    }

    /// Fetch and format the last error.
    pub(crate) fn last_error() -> Self {
        unsafe {
            let last_err = GetLastError();

            let mut msg: PWSTR = ptr::null_mut();
            // the api misuses a pointer as an out-pointer. (* instead of **).
            // we construct a pointer to the storage location of msg and cast it to PWSTR.
            let p_msg = ((&mut msg) as *mut *mut u16) as PWSTR;

            let msg_len = FormatMessageW(
                FORMAT_MESSAGE_FROM_SYSTEM
                    | FORMAT_MESSAGE_IGNORE_INSERTS
                    | FORMAT_MESSAGE_ALLOCATE_BUFFER,
                ptr::null(),     // format-str
                last_err,        // errno
                0,               // languageId
                p_msg,           // message buffer
                0,               // nSize. 0=alloc
                ptr::null_mut(), // args: none
            );

            // Construct a slice from the returned message buffer.
            let s_msg = slice_from_raw_parts(msg, msg_len as usize);
            let os_msg = OsString::from_wide(&*s_msg);

            let err = Self::Print(os_msg.to_string_lossy().to_string());

            LocalFree(msg as HLOCAL);

            err
        }
    }
}

/// Default printer.
pub fn default_printer() -> std::io::Result<String> {
    unsafe {
        let mut buf_len = 0u32;
        if GetDefaultPrinterW(ptr::null_mut(), &mut buf_len as *mut u32) == FALSE {
            let buf_layout = Layout::from_size_align_unchecked(buf_len as usize, align_of::<u16>());
            let buf = alloc_zeroed(buf_layout) as *mut u16;

            if GetDefaultPrinterW(buf, &mut buf_len as *mut u32) == TRUE {
                let os_buf = OsString::from_wide(&*slice_from_raw_parts(buf, buf_len as usize));
                let pr_name = os_buf.to_string_lossy().to_string();

                dealloc(buf as *mut u8, buf_layout);

                Ok(pr_name)
            } else {
                dealloc(buf as *mut u8, buf_layout);

                Err(std::io::Error::new(
                    ErrorKind::Other,
                    PrintError::last_error(),
                ))
            }
        } else {
            unreachable!()
        }
    }
}

#[non_exhaustive]
#[derive(Default, Debug, Clone)]
pub struct Info {
    // -- common --
    /// CUPS: name
    /// WIN: pPrinterName
    pub printer_name: String,
    /// CUPS: printer-uri-supported
    /// WIN: \\\\pserverName\\pPrinterName
    pub printer_uri: String,
    /// CUPS: device-uri
    /// WIN: pPortName
    pub device_uri: String,
    /// CUPS: printer-make-and-model
    /// WIN: pDriverName
    pub driver_name: String,
    /// CUPS: printer-info
    /// WIN: comment
    pub printer_info: String,
    /// CUPS: printer-location
    /// WIN: location
    pub printer_location: String,
    /// CUPS: job-priority
    /// WIN: DefaultPriority
    pub job_priority: u32,

    // -- win-api --
    /// WIN: pServerName
    pub server_name: String,
    /// WIN: pPortName
    pub port_name: String,
    /// WIN: pComment
    pub comment: String,
    /// WIN: pLocation
    pub location: String,
    /// WIN: pSepFile
    pub sep_file: String,
    /// WIN: pPrintProcessor
    pub print_processor: String,
    /// WIN: pDataType
    pub data_type: String,
    /// WIN: pParameters
    pub parameters: String,
    /// WIN: StartTime
    pub start_time: u32,
    /// WIN: UntilTime
    pub until_time: u32,
    /// WIN: cJobs
    pub jobs: u32,
    /// WIN: AveragePPM
    pub average_ppm: u32,

    pub attr_queued: bool,
    pub attr_direct: bool,
    pub attr_default: bool,
    pub attr_network: bool,
    pub attr_shared: bool,
    pub attr_hidden: bool,
    pub attr_local: bool,
    pub attr_enable_devq: bool,
    pub attr_keep_printed_jobs: bool,
    pub attr_do_complete_first: bool,
    pub attr_work_offline: bool,
    pub attr_enable_bidi: bool,
    pub attr_raw_only: bool,
    pub attr_published: bool,
    pub attr_fax: bool,
    pub attr_ts: bool,
    pub attr_pushed_user: bool,
    pub attr_pushed_machine: bool,
    pub attr_machine: bool,
    pub attr_friendly_name: bool,
    pub attr_ts_generic_driver: bool,
    pub attr_per_user: bool,
    pub attr_enterprise_cloud: bool,

    pub status_busy: bool,
    pub status_door_open: bool,
    pub status_error: bool,
    pub status_initializing: bool,
    pub status_io_active: bool,
    pub status_manual_feed: bool,
    pub status_no_toner: bool,
    pub status_not_available: bool,
    pub status_offline: bool,
    pub status_out_of_memory: bool,
    pub status_output_bin_full: bool,
    pub status_page_punt: bool,
    pub status_paper_jam: bool,
    pub status_paper_out: bool,
    pub status_paper_problem: bool,
    pub status_paused: bool,
    pub status_pending_deletion: bool,
    pub status_power_save: bool,
    pub status_printing: bool,
    pub status_processing: bool,
    pub status_server_unknown: bool,
    pub status_toner_low: bool,
    pub status_user_intervention: bool,
    pub status_waiting: bool,
    pub status_warming_up: bool,

    pub device_driver_version: u16,

    pub device_orientation: Option<i16>,
    pub device_paper_size: Option<i16>,
    pub device_paper_length: Option<i16>,
    pub device_paper_width: Option<i16>,
    pub device_paper_scale: Option<i16>,
    pub device_paper_copies: Option<i16>,
    pub device_default_source: Option<i16>,
    pub device_print_quality: Option<i16>,
    pub device_color: Option<i16>,
    pub device_duplex: Option<i16>,
    pub device_y_resolution: Option<i16>,
    pub device_tt_option: Option<i16>,
    pub device_collate: Option<i16>,
    pub device_form_name: Option<String>,
    pub device_n_up: Option<u32>,
    pub device_icm_method: Option<u32>,
    pub device_icm_intent: Option<u32>,
    pub device_media_type: Option<u32>,
    pub device_dither_type: Option<u32>,
}

impl Info {
    /// Try to sum up the status flags.
    pub fn status(&self) -> Status {
        if self.status_busy
            || self.status_printing
            || self.status_initializing
            || self.status_io_active
            || self.status_processing
            || self.status_waiting
            || self.status_warming_up
        {
            Status::Busy
        } else if self.status_offline || self.status_paused || self.status_power_save {
            Status::Stopped
        } else if self.status_toner_low
            || self.status_manual_feed
            || self.status_output_bin_full
            || self.status_paper_out
            || self.status_paper_problem
            || self.status_user_intervention
        {
            Status::Warn
        } else if self.status_door_open
            || self.status_error
            || self.status_no_toner
            || self.status_not_available
            || self.status_out_of_memory
            || self.status_page_punt
            || self.status_paper_jam
            || self.status_pending_deletion
            || self.status_server_unknown
        {
            Status::Error
        } else {
            Status::Idle
        }
    }
}

/// Extended attributes
pub fn printer_attr(pr_name: &str) -> std::io::Result<Info> {
    let pr_name = OsString::from(pr_name)
        .encode_wide()
        .chain(once(0))
        .collect::<Vec<u16>>();

    unsafe {
        let mut pr_handle = 0;
        let mut cb_needed = 0u32;

        if OpenPrinterW(pr_name.as_ptr(), &mut pr_handle as *mut HANDLE, ptr::null()) == 0 {
            dbg!(0);
            return Err(PrintError::last_io_error());
        }

        let result =
            if GetPrinterW(pr_handle, 2, ptr::null_mut(), 0, &mut cb_needed as *mut u32) == 0 {
                let info_layout = Layout::from_size_align_unchecked(
                    cb_needed as usize,
                    align_of::<PRINTER_INFO_2W>(),
                );
                let buf = alloc_zeroed(info_layout);

                let result =
                    if GetPrinterW(pr_handle, 2, buf, cb_needed, &mut cb_needed as *mut u32) != 0 {
                        Ok(copy_info(&*(buf as *mut PRINTER_INFO_2W)))
                    } else {
                        dbg!(1);
                        Err(PrintError::last_io_error())
                    };

                dealloc(buf, info_layout);

                result
            } else {
                unreachable!()
            };

        if ClosePrinter(pr_handle) == FALSE {
            dbg!(2);
            return Err(PrintError::last_io_error());
        }

        result
    }
}

fn copy_info(info: &PRINTER_INFO_2W) -> Info {
    unsafe {
        let mut res = Info::default();

        res.printer_name = extract_wstr(info.pPrinterName);
        res.printer_uri = if res.server_name.is_empty() {
            format!("\\\\.\\{}", res.printer_name).to_string()
        } else {
            format!("\\\\{}\\{}", res.server_name, res.printer_name).to_string()
        };
        res.device_uri = res.port_name.clone();
        res.driver_name = extract_wstr(info.pDriverName);
        res.printer_info = extract_wstr(info.pComment);
        res.printer_location = extract_wstr(info.pLocation);
        res.job_priority = info.DefaultPriority;

        res.server_name = extract_wstr(info.pServerName);
        res.port_name = extract_wstr(info.pPortName);
        res.comment = extract_wstr(info.pComment);
        res.location = extract_wstr(info.pLocation);
        res.sep_file = extract_wstr(info.pSepFile);
        res.print_processor = extract_wstr(info.pPrintProcessor);
        res.data_type = extract_wstr(info.pDatatype);
        res.parameters = extract_wstr(info.pParameters);
        res.start_time = info.StartTime;
        res.until_time = info.UntilTime;
        res.jobs = info.cJobs;
        res.average_ppm = info.AveragePPM;

        res.attr_queued = (info.Attributes & PRINTER_ATTRIBUTE_QUEUED) != 0;
        res.attr_direct = (info.Attributes & PRINTER_ATTRIBUTE_DIRECT) != 0;
        res.attr_default = (info.Attributes & PRINTER_ATTRIBUTE_DEFAULT) != 0;
        res.attr_network = (info.Attributes & PRINTER_ATTRIBUTE_NETWORK) != 0;
        res.attr_shared = (info.Attributes & PRINTER_ATTRIBUTE_SHARED) != 0;
        res.attr_hidden = (info.Attributes & PRINTER_ATTRIBUTE_HIDDEN) != 0;
        res.attr_local = (info.Attributes & PRINTER_ATTRIBUTE_LOCAL) != 0;
        res.attr_enable_devq = (info.Attributes & PRINTER_ATTRIBUTE_ENABLE_DEVQ) != 0;
        res.attr_keep_printed_jobs = (info.Attributes & PRINTER_ATTRIBUTE_KEEPPRINTEDJOBS) != 0;
        res.attr_do_complete_first = (info.Attributes & PRINTER_ATTRIBUTE_DO_COMPLETE_FIRST) != 0;
        res.attr_work_offline = (info.Attributes & PRINTER_ATTRIBUTE_WORK_OFFLINE) != 0;
        res.attr_enable_bidi = (info.Attributes & PRINTER_ATTRIBUTE_ENABLE_BIDI) != 0;
        res.attr_raw_only = (info.Attributes & PRINTER_ATTRIBUTE_RAW_ONLY) != 0;
        res.attr_published = (info.Attributes & PRINTER_ATTRIBUTE_PUBLISHED) != 0;
        res.attr_fax = (info.Attributes & PRINTER_ATTRIBUTE_FAX) != 0;
        res.attr_ts = (info.Attributes & PRINTER_ATTRIBUTE_TS) != 0;
        res.attr_pushed_user = (info.Attributes & PRINTER_ATTRIBUTE_PUSHED_USER) != 0;
        res.attr_pushed_machine = (info.Attributes & PRINTER_ATTRIBUTE_PUSHED_MACHINE) != 0;
        res.attr_machine = (info.Attributes & PRINTER_ATTRIBUTE_MACHINE) != 0;
        res.attr_friendly_name = (info.Attributes & PRINTER_ATTRIBUTE_FRIENDLY_NAME) != 0;
        res.attr_ts_generic_driver = (info.Attributes & PRINTER_ATTRIBUTE_TS_GENERIC_DRIVER) != 0;
        res.attr_per_user = (info.Attributes & PRINTER_ATTRIBUTE_PER_USER) != 0;
        res.attr_enterprise_cloud = (info.Attributes & PRINTER_ATTRIBUTE_ENTERPRISE_CLOUD) != 0;

        res.status_busy = (info.Status & PRINTER_STATUS_BUSY) != 0;
        res.status_door_open = (info.Status & PRINTER_STATUS_DOOR_OPEN) != 0;
        res.status_error = (info.Status & PRINTER_STATUS_ERROR) != 0;
        res.status_initializing = (info.Status & PRINTER_STATUS_INITIALIZING) != 0;
        res.status_io_active = (info.Status & PRINTER_STATUS_IO_ACTIVE) != 0;
        res.status_manual_feed = (info.Status & PRINTER_STATUS_MANUAL_FEED) != 0;
        res.status_no_toner = (info.Status & PRINTER_STATUS_NO_TONER) != 0;
        res.status_not_available = (info.Status & PRINTER_STATUS_NOT_AVAILABLE) != 0;
        res.status_offline = (info.Status & PRINTER_STATUS_OFFLINE) != 0;
        res.status_out_of_memory = (info.Status & PRINTER_STATUS_OUT_OF_MEMORY) != 0;
        res.status_output_bin_full = (info.Status & PRINTER_STATUS_OUTPUT_BIN_FULL) != 0;
        res.status_page_punt = (info.Status & PRINTER_STATUS_PAGE_PUNT) != 0;
        res.status_paper_jam = (info.Status & PRINTER_STATUS_PAPER_JAM) != 0;
        res.status_paper_out = (info.Status & PRINTER_STATUS_PAPER_OUT) != 0;
        res.status_paper_problem = (info.Status & PRINTER_STATUS_PAPER_PROBLEM) != 0;
        res.status_paused = (info.Status & PRINTER_STATUS_PAUSED) != 0;
        res.status_pending_deletion = (info.Status & PRINTER_STATUS_PENDING_DELETION) != 0;
        res.status_power_save = (info.Status & PRINTER_STATUS_POWER_SAVE) != 0;
        res.status_printing = (info.Status & PRINTER_STATUS_PRINTING) != 0;
        res.status_processing = (info.Status & PRINTER_STATUS_PROCESSING) != 0;
        res.status_server_unknown = (info.Status & PRINTER_STATUS_SERVER_UNKNOWN) != 0;
        res.status_toner_low = (info.Status & PRINTER_STATUS_TONER_LOW) != 0;
        res.status_user_intervention = (info.Status & PRINTER_STATUS_USER_INTERVENTION) != 0;
        res.status_waiting = (info.Status & PRINTER_STATUS_WAITING) != 0;
        res.status_warming_up = (info.Status & PRINTER_STATUS_WARMING_UP) != 0;

        let dev_mode = &*info.pDevMode;
        res.device_driver_version = dev_mode.dmDriverVersion;

        if 0 != dev_mode.dmFields & DM_ORIENTATION {
            res.device_orientation = Some(dev_mode.Anonymous1.Anonymous1.dmOrientation);
        }
        if 0 != dev_mode.dmFields & DM_PAPERSIZE {
            res.device_paper_size = Some(dev_mode.Anonymous1.Anonymous1.dmPaperSize);
        }
        if 0 != dev_mode.dmFields & DM_PAPERLENGTH {
            res.device_paper_length = Some(dev_mode.Anonymous1.Anonymous1.dmPaperLength);
        }
        if 0 != dev_mode.dmFields & DM_PAPERWIDTH {
            res.device_paper_width = Some(dev_mode.Anonymous1.Anonymous1.dmPaperWidth);
        }
        if 0 != dev_mode.dmFields & DM_SCALE {
            res.device_paper_scale = Some(dev_mode.Anonymous1.Anonymous1.dmScale);
        }
        if 0 != dev_mode.dmFields & DM_COPIES {
            res.device_paper_copies = Some(dev_mode.Anonymous1.Anonymous1.dmCopies);
        }
        if 0 != dev_mode.dmFields & DM_DEFAULTSOURCE {
            res.device_default_source = Some(dev_mode.Anonymous1.Anonymous1.dmDefaultSource);
        }
        if 0 != dev_mode.dmFields & DM_PRINTQUALITY {
            res.device_print_quality = Some(dev_mode.Anonymous1.Anonymous1.dmPrintQuality);
        }
        if 0 != dev_mode.dmFields & DM_COLOR {
            res.device_color = Some(dev_mode.dmColor);
        }
        if 0 != dev_mode.dmFields & DM_DUPLEX {
            res.device_duplex = Some(dev_mode.dmDuplex);
        }
        if 0 != dev_mode.dmFields & DM_YRESOLUTION {
            res.device_y_resolution = Some(dev_mode.dmYResolution);
        }
        if 0 != dev_mode.dmFields & DM_TTOPTION {
            res.device_tt_option = Some(dev_mode.dmTTOption);
        }
        if 0 != dev_mode.dmFields & DM_COLLATE {
            res.device_collate = Some(dev_mode.dmCollate);
        }

        if 0 != dev_mode.dmFields & DM_FORMNAME {
            let p_form_name = &dev_mode.dmFormName as *const u16;
            let len = min(CCHFORMNAME as usize, wcslen(p_form_name) as usize);
            let slice = slice_from_raw_parts(p_form_name, len);
            let str = OsString::from_wide(&*slice);
            res.device_form_name = Some(str.to_string_lossy().to_string());
        }
        if 0 != dev_mode.dmFields & DM_NUP {
            res.device_n_up = Some(dev_mode.Anonymous2.dmNup);
        }
        if 0 != dev_mode.dmFields & DM_ICMMETHOD {
            res.device_icm_method = Some(dev_mode.dmICMMethod);
        }
        if 0 != dev_mode.dmFields & DM_ICMINTENT {
            res.device_icm_intent = Some(dev_mode.dmICMIntent);
        }
        if 0 != dev_mode.dmFields & DM_MEDIATYPE {
            res.device_media_type = Some(dev_mode.dmMediaType);
        }
        if 0 != dev_mode.dmFields & DM_DITHERTYPE {
            res.device_dither_type = Some(dev_mode.dmDitherType);
        }

        res
    }
}

/// List local printers.
pub fn list_printers() -> std::io::Result<Vec<String>> {
    let mut r = Vec::new();

    unsafe {
        let mut cb_needed = 0u32;
        let mut c_returned = 0u32;

        if EnumPrintersW(
            PRINTER_ENUM_LOCAL,
            ptr::null_mut(),             // name filter
            4,                           // level. type of result struct
            ptr::null_mut(),             // result struct
            0,                           // buf_len. 0 -> return needed as cb_needed
            &mut cb_needed as *mut u32,  // needed bytes
            &mut c_returned as *mut u32, // count of structs
        ) == FALSE
        {
            let info_layout = Layout::from_size_align_unchecked(
                cb_needed as usize,
                align_of::<PRINTER_INFO_4W>(),
            );
            let buf = alloc_zeroed(info_layout);

            if EnumPrintersW(
                PRINTER_ENUM_LOCAL,
                ptr::null_mut(),
                4, // level
                buf,
                cb_needed,
                &mut cb_needed as *mut u32,
                &mut c_returned as *mut u32,
            ) == TRUE
            {
                for i in 0..c_returned as isize {
                    let info = &*(buf as *mut PRINTER_INFO_4W).offset(i);

                    let len_name = wcslen(info.pPrinterName);
                    let slice_name = slice_from_raw_parts(info.pPrinterName, len_name);
                    let os_name = OsString::from_wide(&*slice_name);

                    r.push(os_name.to_string_lossy().to_string())
                }

                dealloc(buf, info_layout);
            } else {
                dealloc(buf, info_layout);

                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    PrintError::last_error(),
                ));
            }
        } else {
            unreachable!()
        }
    }

    Ok(r)
}

/// Printjob data.
#[derive(Clone, Debug)]
pub struct WindowsPrintJob {
    pub printer: HANDLE,
    pub job_id: u32,
}

impl Write for WindowsPrintJob {
    /// Write data to the printer.
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unsafe {
            let mut written = 0u32;

            if 0 != WritePrinter(
                self.printer,
                buf.as_ptr() as *const c_void,
                buf.len() as u32,
                &mut written as *mut u32,
            ) {
                Ok(written as usize)
            } else {
                Err(std::io::Error::new(
                    ErrorKind::Other,
                    PrintError::last_error(),
                ))
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // noop
        Ok(())
    }
}

impl Drop for WindowsPrintJob {
    /// Closes the printjob and sends it to the printer.
    /// Any error is eaten. Use close() directly for error-handling.
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl WindowsPrintJob {
    /// Starts a printjob.
    pub fn new(pr_name: &str, doc_name: &str) -> Result<WindowsPrintJob, std::io::Error> {
        Self::new_with(pr_name, doc_name, &JobParam::default())
    }

    pub fn new_with(pr_name: &str, doc_name: &str, param: &JobParam) -> std::io::Result<Self> {
        let mut print = WindowsPrintJob {
            printer: 0,
            job_id: 0,
        };

        unsafe {
            let data_format = match param.data_format {
                Format::Raw => "RAW",
                Format::RawFFAppended => "RAW [FF appended]",
                Format::RawFFAuto => "RAW [FF auto]",
                Format::NtEmf1_003 => "NT EMF 1.003",
                Format::NtEmf1_006 => "NT EMF 1.006",
                Format::NtEmf1_007 => "NT EMF 1.007",
                Format::NtEmf1_008 => "NT EMF 1.008",
                Format::Text => "TEXT",
                Format::XpsPass => "XPS_PASS",
                Format::Xps2Gdi => "XPS2GDI",
            };
            let mut data_format = OsString::from(data_format)
                .encode_wide()
                .chain(once(0))
                .collect::<Vec<u16>>();

            let pr_name = OsString::from(pr_name)
                .encode_wide()
                .chain(once(0))
                .collect::<Vec<u16>>();

            let mut devmode = DEVMODEW {
                dmDeviceName: [0; 32],
                dmSpecVersion: DM_SPECVERSION as u16,
                dmDriverVersion: 0,
                dmSize: size_of::<DEVMODEW>() as u16,
                dmDriverExtra: 0,
                dmFields: 0,
                Anonymous1: DEVMODEW_0 {
                    Anonymous1: DEVMODEW_0_0 {
                        dmOrientation: 0,
                        dmPaperSize: 0,
                        dmPaperLength: 0,
                        dmPaperWidth: 0,
                        dmScale: 0,
                        dmCopies: 0,
                        dmDefaultSource: 0,
                        dmPrintQuality: 0,
                    },
                },
                dmColor: 0,
                dmDuplex: 0,
                dmYResolution: 0,
                dmTTOption: 0,
                dmCollate: 0,
                dmFormName: [0; 32],
                dmLogPixels: 0,
                dmBitsPerPel: 0,
                dmPelsWidth: 0,
                dmPelsHeight: 0,
                Anonymous2: DEVMODEW_1 { dmNup: 0 },
                dmDisplayFrequency: 0,
                dmICMMethod: 0,
                dmICMIntent: 0,
                dmMediaType: 0,
                dmDitherType: 0,
                dmReserved1: 0,
                dmReserved2: 0,
                dmPanningWidth: 0,
                dmPanningHeight: 0,
            };

            if let Some(copies) = param.copies {
                devmode.dmFields |= DM_COPIES;
                devmode.Anonymous1.Anonymous1.dmCopies = copies as i16;
            }
            if let Some(paper_size) = param.paper_size {
                let paper_size = match paper_size {
                    PaperSize::Numeric(n) => n,
                    _ => paper_size.discriminant() as i16,
                };
                devmode.dmFields |= DM_PAPERSIZE;
                devmode.Anonymous1.Anonymous1.dmPaperSize = paper_size;
            }
            if let Some(paper_source) = param.paper_source {
                let paper_source = match paper_source {
                    PaperSource::Numeric(v) => v,
                    _ => paper_source.discriminant() as i16,
                };
                devmode.dmFields |= DM_DEFAULTSOURCE;
                devmode.Anonymous1.Anonymous1.dmDefaultSource = paper_source;
            }
            if let Some(paper_type) = param.paper_type {
                let paper_type = match paper_type {
                    PaperType::Numeric(v) => v,
                    _ => paper_type.discriminant(),
                };
                devmode.dmFields |= DM_MEDIATYPE;
                devmode.dmMediaType = paper_type;
            }
            if let Some(orientation) = param.orientation {
                let orientation = match orientation {
                    Orientation::Numeric(v) => v,
                    _ => orientation.discriminant() as i16,
                };
                devmode.dmFields |= DM_ORIENTATION;
                devmode.Anonymous1.Anonymous1.dmOrientation = orientation;
            }
            if let Some(color) = param.color {
                let color = match color {
                    ColorMode::Numeric(v) => v,
                    _ => color.discriminant(),
                };
                devmode.dmFields |= DM_COLOR;
                devmode.dmColor = color;
            }
            if let Some(quality) = param.quality {
                let quality = match quality {
                    Quality::Numeric(v) => v,
                    _ => quality.discriminant() as i16,
                };
                devmode.dmFields |= DM_PRINTQUALITY;
                devmode.Anonymous1.Anonymous1.dmPrintQuality = quality;
            }
            if let Some(duplex) = param.duplex {
                let duplex = match duplex {
                    Duplex::Numeric(v) => v,
                    _ => duplex.discriminant(),
                };
                devmode.dmFields |= DM_DUPLEX;
                devmode.dmDuplex = duplex;
            }
            if let Some(paper_length) = param.paper_length {
                devmode.dmFields |= DM_PAPERLENGTH;
                devmode.Anonymous1.Anonymous1.dmPaperLength = paper_length;
            }
            if let Some(paper_width) = param.paper_width {
                devmode.dmFields |= DM_PAPERWIDTH;
                devmode.Anonymous1.Anonymous1.dmPaperWidth = paper_width;
            }
            if let Some(scale) = param.scale {
                devmode.dmFields |= DM_SCALE;
                devmode.Anonymous1.Anonymous1.dmScale = scale;
            }
            if let Some(y_resolution) = param.y_resolution {
                devmode.dmFields |= DM_YRESOLUTION;
                devmode.dmYResolution = y_resolution;
            }
            if let Some(tt_option) = param.tt_option {
                let tt_option = match tt_option {
                    TrueType::Numeric(v) => v,
                    _ => tt_option.discriminant(),
                };
                devmode.dmFields |= DM_TTOPTION;
                devmode.dmTTOption = tt_option;
            }
            if let Some(collate) = param.collate {
                let collate = match collate {
                    Collate::Numeric(v) => v,
                    _ => collate.discriminant(),
                };
                devmode.dmFields |= DM_COLLATE;
                devmode.dmCollate = collate;
            }

            let defaults = PRINTER_DEFAULTSW {
                pDatatype: data_format.as_mut_ptr() as PWSTR,
                pDevMode: &mut devmode as *mut DEVMODEW,
                DesiredAccess: PRINTER_ACCESS_USE,
            };

            if 0 != OpenPrinterW(
                pr_name.as_ptr() as PCWSTR,
                &mut print.printer as *mut HANDLE,
                &defaults as *const PRINTER_DEFAULTSW,
            ) {
                let mut doc_name = OsString::from(doc_name)
                    .encode_wide()
                    .chain(once(0))
                    .collect::<Vec<u16>>();
                let doc_info = DOC_INFO_1W {
                    pDocName: doc_name.as_mut_ptr(),
                    pOutputFile: ptr::null_mut(),
                    pDatatype: data_format.as_mut_ptr() as PWSTR,
                };

                print.job_id = StartDocPrinterW(print.printer, 1, &doc_info as *const DOC_INFO_1W);
                if print.job_id != 0 {
                    Ok(print)
                } else {
                    ClosePrinter(print.printer);
                    Err(std::io::Error::new(
                        ErrorKind::Other,
                        PrintError::last_error(),
                    ))
                }
            } else {
                Err(std::io::Error::new(
                    ErrorKind::Other,
                    PrintError::last_error(),
                ))
            }
        }
    }

    /// Close the printjob.
    pub fn close(&mut self) -> Result<(), std::io::Error> {
        unsafe {
            if self.printer == 0 {
                return Ok(());
            }

            if 0 != EndDocPrinter(self.printer) {
                if 0 != ClosePrinter(self.printer) {
                    self.printer = 0;
                    Ok(())
                } else {
                    Err(std::io::Error::new(
                        ErrorKind::Other,
                        PrintError::last_error(),
                    ))
                }
            } else {
                Err(std::io::Error::new(
                    ErrorKind::Other,
                    PrintError::last_error(),
                ))
            }
        }
    }

    /// Start a new page. More a hint to the spooling system, wherever it
    /// displays a page count.
    pub fn start_page(&self) -> Result<(), std::io::Error> {
        unsafe {
            if 0 != StartPagePrinter(self.printer) {
                Ok(())
            } else {
                Err(std::io::Error::new(
                    ErrorKind::Other,
                    PrintError::last_error(),
                ))
            }
        }
    }

    /// End a page.
    pub fn end_page(&self) -> Result<(), std::io::Error> {
        unsafe {
            if 0 != EndPagePrinter(self.printer) {
                Ok(())
            } else {
                Err(std::io::Error::from(ErrorKind::Other))
            }
        }
    }
}

fn extract_wstr(value: PWSTR) -> String {
    unsafe {
        if !value.is_null() {
            let len = wcslen(value);
            let slice = slice_from_raw_parts(value, len);

            let os_str = OsString::from_wide(&*slice);

            os_str.to_string_lossy().to_string()
        } else {
            String::default()
        }
    }
}
