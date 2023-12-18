//!
//!
//! References:
//! https://www.rfc-editor.org/rfc/rfc8011
//! https://www.cups.org
//! https://stackoverflow.com/questions/44687154/more-complete-list-of-cups-printer-state-reasons

use std::ffi::{c_char, c_int, CStr, CString};
use std::io::{ErrorKind, Write};
use std::ptr::{self, slice_from_raw_parts};
use std::str::FromStr;

use cups_sys::{
    cupsAddOption, cupsCreateJob, cupsFinishDocument, cupsFreeDests, cupsGetDests,
    cupsGetNamedDest, cupsLastErrorString, cupsStartDocument, cupsWriteRequestData, CUPS_COPIES,
    CUPS_FINISHINGS, CUPS_FINISHINGS_BIND, CUPS_FINISHINGS_COVER, CUPS_FINISHINGS_FOLD,
    CUPS_FINISHINGS_NONE, CUPS_FINISHINGS_PUNCH, CUPS_FINISHINGS_STAPLE, CUPS_FINISHINGS_TRIM,
    CUPS_MEDIA_SOURCE, CUPS_MEDIA_SOURCE_AUTO, CUPS_MEDIA_SOURCE_MANUAL, CUPS_MEDIA_TYPE,
    CUPS_MEDIA_TYPE_AUTO, CUPS_MEDIA_TYPE_ENVELOPE, CUPS_MEDIA_TYPE_LABELS,
    CUPS_MEDIA_TYPE_LETTERHEAD, CUPS_MEDIA_TYPE_PHOTO, CUPS_MEDIA_TYPE_PHOTO_GLOSSY,
    CUPS_MEDIA_TYPE_PHOTO_MATTE, CUPS_MEDIA_TYPE_PLAIN, CUPS_MEDIA_TYPE_TRANSPARENCY,
    CUPS_NUMBER_UP, CUPS_ORIENTATION, CUPS_ORIENTATION_LANDSCAPE, CUPS_PRINT_COLOR_MODE,
    CUPS_PRINT_COLOR_MODE_AUTO, CUPS_PRINT_COLOR_MODE_COLOR, CUPS_PRINT_COLOR_MODE_MONOCHROME,
    CUPS_PRINT_QUALITY, CUPS_PRINT_QUALITY_DRAFT, CUPS_PRINT_QUALITY_HIGH,
    CUPS_PRINT_QUALITY_NORMAL, CUPS_SIDES, CUPS_SIDES_ONE_SIDED, CUPS_SIDES_TWO_SIDED_LANDSCAPE,
    CUPS_SIDES_TWO_SIDED_PORTRAIT, CUPS_FORMAT_COMMAND, CUPS_FORMAT_TEXT, CUPS_FORMAT_AUTO, CUPS_FORMAT_JPEG, CUPS_FORMAT_PDF, CUPS_FORMAT_POSTSCRIPT,
};
use cups_sys::{cups_dest_t, cups_option_t};
use cups_sys::{
    http_status_e_HTTP_STATUS_CONTINUE as HTTP_STATUS_CONTINUE, http_t,
    ipp_status_e_IPP_STATUS_OK as IPP_STATUS_OK, CUPS_FORMAT_RAW, CUPS_MEDIA, CUPS_MEDIA_3X5,
    CUPS_MEDIA_4X6, CUPS_MEDIA_5X7, CUPS_MEDIA_8X10, CUPS_MEDIA_A3, CUPS_MEDIA_A4, CUPS_MEDIA_A5,
    CUPS_MEDIA_A6, CUPS_MEDIA_ENV10, CUPS_MEDIA_ENVDL, CUPS_MEDIA_LEGAL, CUPS_MEDIA_LETTER,
    CUPS_MEDIA_PHOTO_L, CUPS_MEDIA_SUPERBA3, CUPS_MEDIA_TABLOID,
};

use crate::{JobParam, PrintError};

impl PrintError {
    pub(crate) fn io_error(e: PrintError) -> std::io::Error {
        std::io::Error::new(ErrorKind::Other, e)
    }

    pub(crate) fn last_io_error() -> std::io::Error {
        std::io::Error::new(ErrorKind::Other, PrintError::last_error())
    }

    /// Fetch the last error.
    pub(crate) fn last_error() -> Self {
        unsafe {
            let e = CStr::from_ptr(cupsLastErrorString());
            PrintError::Print(e.to_string_lossy().to_string())
        }
    }
}

/// Default printer.
pub fn default_printer() -> std::io::Result<String> {
    unsafe {
        let mut cups_dest: *mut cups_dest_t = ptr::null_mut::<cups_dest_t>();
        let pcups_dest = (&mut cups_dest) as *mut *mut cups_dest_t;

        let n_dests = cupsGetDests(pcups_dest);

        for i in 0isize..n_dests as isize {
            let cur_dest = cups_dest.offset(i);

            if (*cur_dest).is_default == 1 {
                let c_name = CStr::from_ptr((*cur_dest).name);
                let name = c_name.to_string_lossy().to_string();
                cupsFreeDests(n_dests, cups_dest);
                return Ok(name);
            }
        }

        cupsFreeDests(n_dests, cups_dest);

        Err(PrintError::io_error(PrintError::NoDefaultPrinter))
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Format {
    #[default]
    Raw,
    Auto,
    Command,
    Jpeg,
    Pdf,
    Postscript,
    Text
}

#[derive(Default, Debug, Clone, Copy)]
pub enum PaperSize {
    Size3x5,
    Size4x6,
    Size5x7,
    Size8x10,
    A3,
    A4,
    A5,
    A6,
    Env10,
    EnvDl,
    Legal,
    #[default]
    Letter,
    PhotoL,
    SuperBA3,
    Tabloid,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum PaperSource {
    #[default]
    Auto,
    Manual,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Finishings {
    #[default]
    None,
    Staple,
    Punch,
    Cover,
    Bind,
    Fold,
    Trim,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum ColorMode {
    #[default]
    Auto,
    Monochrome,
    Color,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Duplex {
    #[default]
    Simplex,
    TwoSidedPortrait,
    TwoSidedLandscape,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum PaperType {
    #[default]
    Auto,
    Envelope,
    Labels,
    Letterhead,
    Photo,
    PhotoGlossy,
    PhotoMatte,
    Plain,
    Transparency,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Quality {
    Draft,
    #[default]
    Normal,
    High,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum PrinterState {
    #[default]
    Idle,
    Printing,
    Stopped,
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

    // -- cups --
    pub printer_instance: Option<String>,
    pub job_cancel_after: u32,
    pub job_hold_until: String,
    pub job_sheets: String,
    pub marker_change_time: u64,

    pub copies: u32,
    pub finishings: Finishings,
    pub number_up: u32,
    pub print_color_mode: ColorMode,

    pub printer_is_accepting_jobs: bool,
    pub printer_is_shared: bool,
    pub printer_is_temporary: bool,
    pub printer_type: u32,

    pub printer_state: PrinterState,
    pub printer_state_change_time: u64,

    pub state_reason_none: bool,
    pub state_reason_other: bool,
    pub state_reason_developer_low: bool,
    pub state_reason_door_open: bool,
    pub state_reason_fuser_over_temp: bool,
    pub state_reason_fuser_under_temp: bool,
    pub state_reason_input_tray_missing: bool,
    pub state_reason_interlock_open: bool,
    pub state_reason_interpreter_resource_unavailable: bool,
    pub state_reason_marker_supply_empty: bool,
    pub state_reason_marker_supply_low: bool,
    pub state_reason_waste_almost_full: bool,
    pub state_reason_waste_full: bool,
    pub state_reason_media_empty: bool,
    pub state_reason_media_jam: bool,
    pub state_reason_media_low: bool,
    pub state_reason_media_needed: bool,
    pub state_reason_moving_to_paused: bool,
    pub state_reason_opc_life_over: bool,
    pub state_reason_opc_near_eol: bool,
    pub state_reason_output_area_almost_full: bool,
    pub state_reason_output_area_full: bool,
    pub state_reason_output_tray_missing: bool,
    pub state_reason_paused: bool,
    pub state_reason_shutdown: bool,
    pub state_reason_spool_area_full: bool,
    pub state_reason_stopped_partly: bool,
    pub state_reason_stopping: bool,
    pub state_reason_timed_out: bool,
    pub state_reason_toner_empty: bool,
    pub state_reason_toner_low: bool,
    pub state_reason_connection_to_device: bool,
    pub state_reason_offline_report: bool,
    pub state_reason_insecure_filter_warning: bool,
    pub state_reason_missing_filter_warning: bool,
    pub state_reason_remote_aborted: bool,
    pub state_reason_remote_canceled: bool,
    pub state_reason_remote_completed: bool,
    pub state_reason_remote_pending: bool,
    pub state_reason_remote_pending_held: bool,
    pub state_reason_remote_processing: bool,
    pub state_reason_remote_stopped: bool,
    pub state_reason_waiting_for_job_completed: bool,
}

pub fn printer_attr(pr_name: &str) -> std::io::Result<Info> {
    unsafe {
        let cups_dest = cupsGetNamedDest(
            ptr::null_mut::<http_t>(),
            pr_name.as_ptr() as *const c_char,
            ptr::null(),
        );
        if !cups_dest.is_null() {
            let cups_dest = &mut *cups_dest;

            let mut result = Info::default();

            result.printer_name = CStr::from_ptr(cups_dest.name).to_string_lossy().to_string();
            result.printer_instance = if !cups_dest.instance.is_null() {
                Some(
                    CStr::from_ptr(cups_dest.instance)
                        .to_string_lossy()
                        .to_string(),
                )
            } else {
                None
            };

            result.printer_uri = find_option(
                "printer-uri-supported",
                cups_dest.num_options,
                cups_dest.options,
            );

            result.device_uri = find_option("device-uri", cups_dest.num_options, cups_dest.options);

            result.driver_name = find_option(
                "printer-make-and-model",
                cups_dest.num_options,
                cups_dest.options,
            );

            result.printer_info =
                find_option("printer-info", cups_dest.num_options, cups_dest.options);

            result.printer_location =
                find_option("printer-location", cups_dest.num_options, cups_dest.options);

            result.job_priority =
                find_num_option("job-priority", cups_dest.num_options, cups_dest.options)?;

            result.job_cancel_after =
                find_num_option("job-cancel-after", cups_dest.num_options, cups_dest.options)?;

            result.job_hold_until =
                find_option("job-hold-until", cups_dest.num_options, cups_dest.options);

            result.job_sheets = find_option("job-sheets", cups_dest.num_options, cups_dest.options);

            result.marker_change_time = find_num_option(
                "marker-change-time",
                cups_dest.num_options,
                cups_dest.options,
            )?;

            let opt = find_option("print-color-mode", cups_dest.num_options, cups_dest.options);
            result.print_color_mode = option_map(
                &opt,
                &[
                    ("auto", ColorMode::Auto),
                    ("monochrome", ColorMode::Monochrome),
                    ("color", ColorMode::Color),
                ],
            );

            result.copies = find_num_option("copies", cups_dest.num_options, cups_dest.options)?;

            let opt = find_option("finishings", cups_dest.num_options, cups_dest.options);
            result.finishings = option_map(
                &opt,
                &[
                    ("3", Finishings::None),
                    ("4", Finishings::Staple),
                    ("5", Finishings::Punch),
                    ("6", Finishings::Cover),
                    ("7", Finishings::Bind),
                    ("10", Finishings::Fold),
                    ("11", Finishings::Trim),
                ],
            );

            result.number_up =
                find_num_option("number-up", cups_dest.num_options, cups_dest.options)?;

            let opt = find_option(
                "printer-is-accepting-jobs",
                cups_dest.num_options,
                cups_dest.options,
            );
            result.printer_is_accepting_jobs =
                option_map(&opt, &[("true", true), ("false", false)]);

            let opt = find_option(
                "printer-is-shared",
                cups_dest.num_options,
                cups_dest.options,
            );
            result.printer_is_shared = option_map(&opt, &[("true", true), ("false", false)]);

            let opt = find_option(
                "printer-is-temporary",
                cups_dest.num_options,
                cups_dest.options,
            );
            result.printer_is_temporary = option_map(&opt, &[("true", true), ("false", false)]);

            let opt = find_option("printer-state", cups_dest.num_options, cups_dest.options);
            result.printer_state = option_map(
                &opt,
                &[
                    ("3", PrinterState::Idle),
                    ("4", PrinterState::Printing),
                    ("5", PrinterState::Stopped),
                ],
            );

            result.printer_state_change_time = find_num_option(
                "printer-state-change-time",
                cups_dest.num_options,
                cups_dest.options,
            )?;

            let opt = find_option(
                "printer-state-reasons",
                cups_dest.num_options,
                cups_dest.options,
            );
            match opt.as_str() {
                "none" => result.state_reason_none = true,
                "other" => result.state_reason_other = true,
                "developer-low" => result.state_reason_developer_low = true,
                "door-open" => result.state_reason_door_open = true,
                "fuser-over-temp" => result.state_reason_fuser_over_temp = true,
                "fuser-under-temp" => result.state_reason_fuser_under_temp = true,
                "input-tray-missing" => result.state_reason_input_tray_missing = true,
                "interlock-open" => result.state_reason_interlock_open = true,
                "interpreter-resource-unavailable" => {
                    result.state_reason_interpreter_resource_unavailable = true
                }
                "marker-supply-empty" => result.state_reason_marker_supply_empty = true,
                "marker-supply-low" => result.state_reason_marker_supply_low = true,
                "waste-almost-full" => result.state_reason_waste_almost_full = true,
                "waste-full" => result.state_reason_waste_full = true,
                "media-empty" => result.state_reason_media_empty = true,
                "media-jam" => result.state_reason_media_jam = true,
                "media-low" => result.state_reason_media_low = true,
                "media-needed" => result.state_reason_media_needed = true,
                "moving-to-paused" => result.state_reason_moving_to_paused = true,
                "opc-life-over" => result.state_reason_opc_life_over = true,
                "opc-near-eol" => result.state_reason_opc_near_eol = true,
                "output-area-almost-full" => result.state_reason_output_area_almost_full = true,
                "output-area-full" => result.state_reason_output_area_full = true,
                "output-tray-missing" => result.state_reason_output_tray_missing = true,
                "paused" => result.state_reason_paused = true,
                "shutdown" => result.state_reason_shutdown = true,
                "spool-area-full" => result.state_reason_spool_area_full = true,
                "stopped-partly" => result.state_reason_stopped_partly = true,
                "stopping" => result.state_reason_stopping = true,
                "timed-out" => result.state_reason_timed_out = true,
                "toner-empty" => result.state_reason_toner_empty = true,
                "toner-low" => result.state_reason_toner_low = true,
                "connection-to-device" => result.state_reason_connection_to_device = true,
                "offline-report" => result.state_reason_offline_report = true,
                "insecure-filter-warning" => result.state_reason_insecure_filter_warning = true,
                "missing-filter-warning" => result.state_reason_missing_filter_warning = true,
                "remote-aborted" => result.state_reason_remote_aborted = true,
                "remote-canceled" => result.state_reason_remote_canceled = true,
                "remote-completed" => result.state_reason_remote_completed = true,
                "remote-pending" => result.state_reason_remote_pending = true,
                "remote-pending-held" => result.state_reason_remote_pending_held = true,
                "remote-processing" => result.state_reason_remote_processing = true,
                "remote-stopped" => result.state_reason_remote_stopped = true,
                "waiting-for-job-completed" => result.state_reason_waiting_for_job_completed = true,
                _ => {}
            }

            result.printer_type =
                find_num_option("printer-type", cups_dest.num_options, cups_dest.options)?;

            cupsFreeDests(1, cups_dest);

            Ok(result)
        } else {
            Err(PrintError::io_error(PrintError::NotFound))
        }
    }
}

// future: supported and more actual values
//
//            let dinfo = cupsCopyDestInfo(ptr::null_mut::<http_t>(), cups_dest as *mut cups_dest_t);
//
//            let copies = cupsCheckDestSupported(
//                ptr::null_mut::<http_t>(),
//                cups_dest as *mut cups_dest_t,
//                dinfo,
//                // CString::new("copies").expect("copies").as_bytes_with_nul().as_ptr() as *const c_char,
//                CUPS_COPIES.as_ptr() as *const c_char,
//                ptr::null(),
//            ) != 0;
//            println!("copies={}", copies);
//
//            let copies = cupsFindDestReady(
//                ptr::null_mut::<http_t>(),
//                cups_dest as *mut cups_dest_t,
//                dinfo,
//                CUPS_COPIES.as_ptr() as *const c_char,
//            );
//            let copies = ippGetInteger(copies, 0);
//            println!("copies={}", copies);
//
//            cupsFreeDestInfo(dinfo);

/// Find a specific option.
/// Assumes options are ordered by name.
fn find_option(name: &str, n: i32, options: *const cups_option_t) -> String {
    unsafe {
        let options = &*slice_from_raw_parts(options, n as usize);

        if let Ok(n) = options.binary_search_by(|v| {
            let c_name = CStr::from_ptr(v.name);
            c_name.to_bytes().cmp(name.as_bytes())
        }) {
            let c_val = CStr::from_ptr(options[n].value);
            c_val.to_string_lossy().to_string()
        } else {
            String::default()
        }
    }
}

/// Find a specific option and converts it to a number via parse().
/// Assumes options are ordered by name.
fn find_num_option<T>(
    name: &str,
    n: i32,
    options: *const cups_option_t,
) -> Result<T, std::io::Error>
where
    T: FromStr + Default,
    <T as FromStr>::Err: Into<PrintError>,
{
    unsafe {
        let options = &*slice_from_raw_parts(options, n as usize);

        if let Ok(n) = options.binary_search_by(|v| {
            let c_name = CStr::from_ptr(v.name);
            c_name.to_bytes().cmp(name.as_bytes())
        }) {
            let c_val = CStr::from_ptr(options[n].value);
            match (*c_val).to_string_lossy().parse() {
                Ok(v) => Ok(v),
                Err(e) => Err(PrintError::io_error(e.into())),
            }
        } else {
            Ok(T::default())
        }
    }
}

/// Find a specific option and map it to some T.
/// The first value is treated as default value.
fn option_map<T: Clone>(s: &str, opt: &[(&str, T)]) -> T {
    for v in opt {
        if s == v.0 {
            return v.1.clone();
        }
    }
    opt[0].1.clone() // todo: is this ok?
}

/// List installed printers.
pub fn list_printers() -> std::io::Result<Vec<String>> {
    let mut r = Vec::new();

    unsafe {
        let mut cups_dest: *mut cups_dest_t = ptr::null_mut::<cups_dest_t>();
        let pcups_dest = (&mut cups_dest) as *mut *mut cups_dest_t;

        let n_dests = cupsGetDests(pcups_dest);

        for i in 0isize..n_dests as isize {
            let cur_dest = cups_dest.offset(i);

            let c_name = CStr::from_ptr((*cur_dest).name);
            r.push(String::from_utf8_lossy(c_name.to_bytes()).to_string());
        }

        cupsFreeDests(n_dests, cups_dest);
    }

    Ok(r)
}

/// Printjob data.
#[derive(Clone, Debug)]
pub(crate) struct LinuxPrintJob {
    pr_name: CString,
    doc_name: CString,
    job_id: c_int,
}

impl Write for LinuxPrintJob {
    /// Write bytes to the printer.
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unsafe {
            if cupsWriteRequestData(
                ptr::null_mut::<http_t>(),
                buf.as_ptr() as *const c_char,
                buf.len(),
            ) != HTTP_STATUS_CONTINUE
            {
                Err(PrintError::last_io_error())
            } else {
                Ok(buf.len())
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for LinuxPrintJob {
    /// Closes the printjob and sends it to the printer.
    /// Any error is eaten. Use close() directly for error-handling.
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl LinuxPrintJob {
    /// Starts a printjob.
    pub fn new(pr_name: &str, doc_name: &str) -> std::io::Result<Self> {
        Self::new_with(pr_name, doc_name, &JobParam::default())
    }

    pub fn new_with(pr_name: &str, doc_name: &str, param: &JobParam) -> std::io::Result<Self> {
        let mut job = LinuxPrintJob {
            pr_name: CString::new(pr_name)?,
            doc_name: CString::new(doc_name)?,
            job_id: 0,
        };

        unsafe {
            let mut options = ptr::null_mut::<cups_option_t>();
            let p_options = (&mut options) as *mut *mut cups_option_t;
            let mut num_options = 0;

            if let Some(copies) = param.copies {
                let copies = CString::new(copies.to_string())?;
                num_options = cupsAddOption(
                    CUPS_COPIES.as_ptr() as *const c_char,
                    copies.as_ptr(),
                    num_options,
                    p_options,
                );
            }
            if let Some(finishings) = param.finishings {
                let finishings = match finishings {
                    Finishings::None => CUPS_FINISHINGS_NONE.as_ptr(),
                    Finishings::Staple => CUPS_FINISHINGS_STAPLE.as_ptr(),
                    Finishings::Punch => CUPS_FINISHINGS_PUNCH.as_ptr(),
                    Finishings::Cover => CUPS_FINISHINGS_COVER.as_ptr(),
                    Finishings::Bind => CUPS_FINISHINGS_BIND.as_ptr(),
                    Finishings::Fold => CUPS_FINISHINGS_FOLD.as_ptr(),
                    Finishings::Trim => CUPS_FINISHINGS_TRIM.as_ptr(),
                };
                num_options = cupsAddOption(
                    CUPS_FINISHINGS.as_ptr() as *const c_char,
                    finishings as *const c_char,
                    num_options,
                    p_options,
                );
            }
            if let Some(paper_size) = param.paper_size {
                let paper_size = match paper_size {
                    PaperSize::Size3x5 => CUPS_MEDIA_3X5.as_ptr(),
                    PaperSize::Size4x6 => CUPS_MEDIA_4X6.as_ptr(),
                    PaperSize::Size5x7 => CUPS_MEDIA_5X7.as_ptr(),
                    PaperSize::Size8x10 => CUPS_MEDIA_8X10.as_ptr(),
                    PaperSize::A3 => CUPS_MEDIA_A3.as_ptr(),
                    PaperSize::A4 => CUPS_MEDIA_A4.as_ptr(),
                    PaperSize::A5 => CUPS_MEDIA_A5.as_ptr(),
                    PaperSize::A6 => CUPS_MEDIA_A6.as_ptr(),
                    PaperSize::Env10 => CUPS_MEDIA_ENV10.as_ptr(),
                    PaperSize::EnvDl => CUPS_MEDIA_ENVDL.as_ptr(),
                    PaperSize::Legal => CUPS_MEDIA_LEGAL.as_ptr(),
                    PaperSize::Letter => CUPS_MEDIA_LETTER.as_ptr(),
                    PaperSize::PhotoL => CUPS_MEDIA_PHOTO_L.as_ptr(),
                    PaperSize::SuperBA3 => CUPS_MEDIA_SUPERBA3.as_ptr(),
                    PaperSize::Tabloid => CUPS_MEDIA_TABLOID.as_ptr(),
                };
                num_options = cupsAddOption(
                    CUPS_MEDIA.as_ptr() as *const c_char,
                    paper_size as *const c_char,
                    num_options,
                    p_options,
                );
            }
            if let Some(paper_source) = param.paper_source {
                let paper_source = match paper_source {
                    PaperSource::Auto => CUPS_MEDIA_SOURCE_AUTO.as_ptr(),
                    PaperSource::Manual => CUPS_MEDIA_SOURCE_MANUAL.as_ptr(),
                };
                num_options = cupsAddOption(
                    CUPS_MEDIA_SOURCE.as_ptr() as *const c_char,
                    paper_source as *const c_char,
                    num_options,
                    p_options,
                );
            }
            if let Some(paper_type) = param.paper_type {
                let paper_type = match paper_type {
                    PaperType::Auto => CUPS_MEDIA_TYPE_AUTO.as_ptr(),
                    PaperType::Envelope => CUPS_MEDIA_TYPE_ENVELOPE.as_ptr(),
                    PaperType::Labels => CUPS_MEDIA_TYPE_LABELS.as_ptr(),
                    PaperType::Letterhead => CUPS_MEDIA_TYPE_LETTERHEAD.as_ptr(),
                    PaperType::Photo => CUPS_MEDIA_TYPE_PHOTO.as_ptr(),
                    PaperType::PhotoGlossy => CUPS_MEDIA_TYPE_PHOTO_GLOSSY.as_ptr(),
                    PaperType::PhotoMatte => CUPS_MEDIA_TYPE_PHOTO_MATTE.as_ptr(),
                    PaperType::Plain => CUPS_MEDIA_TYPE_PLAIN.as_ptr(),
                    PaperType::Transparency => CUPS_MEDIA_TYPE_TRANSPARENCY.as_ptr(),
                };
                num_options = cupsAddOption(
                    CUPS_MEDIA_TYPE.as_ptr() as *const c_char,
                    paper_type as *const c_char,
                    num_options,
                    p_options,
                );
            }
            if let Some(number_up) = param.number_up {
                let number_up = CString::new(number_up.to_string())?;
                num_options = cupsAddOption(
                    CUPS_NUMBER_UP.as_ptr() as *const c_char,
                    number_up.as_ptr(),
                    num_options,
                    p_options,
                );
            }
            if let Some(orientation) = param.orientation {
                let orientation = match orientation {
                    Orientation::Portrait => CUPS_ORIENTATION_LANDSCAPE.as_ptr(),
                    Orientation::Landscape => CUPS_ORIENTATION_LANDSCAPE.as_ptr(),
                };
                num_options = cupsAddOption(
                    CUPS_ORIENTATION.as_ptr() as *const c_char,
                    orientation as *const c_char,
                    num_options,
                    p_options,
                );
            }
            if let Some(color) = param.color {
                let color = match color {
                    ColorMode::Auto => CUPS_PRINT_COLOR_MODE_AUTO.as_ptr(),
                    ColorMode::Monochrome => CUPS_PRINT_COLOR_MODE_COLOR.as_ptr(),
                    ColorMode::Color => CUPS_PRINT_COLOR_MODE_MONOCHROME.as_ptr(),
                };
                num_options = cupsAddOption(
                    CUPS_PRINT_COLOR_MODE.as_ptr() as *const c_char,
                    color as *const c_char,
                    num_options,
                    p_options,
                );
            }
            if let Some(quality) = param.quality {
                let quality = match quality {
                    Quality::Draft => CUPS_PRINT_QUALITY_DRAFT.as_ptr(),
                    Quality::Normal => CUPS_PRINT_QUALITY_NORMAL.as_ptr(),
                    Quality::High => CUPS_PRINT_QUALITY_HIGH.as_ptr(),
                };
                num_options = cupsAddOption(
                    CUPS_PRINT_QUALITY.as_ptr() as *const c_char,
                    quality as *const c_char,
                    num_options,
                    p_options,
                );
            }
            if let Some(duplex) = param.duplex {
                let duplex = match duplex {
                    Duplex::Simplex => CUPS_SIDES_ONE_SIDED.as_ptr(),
                    Duplex::TwoSidedPortrait => CUPS_SIDES_TWO_SIDED_PORTRAIT.as_ptr(),
                    Duplex::TwoSidedLandscape => CUPS_SIDES_TWO_SIDED_LANDSCAPE.as_ptr(),
                };
                num_options = cupsAddOption(
                    CUPS_SIDES.as_ptr() as *const c_char,
                    duplex as *const c_char,
                    num_options,
                    p_options,
                );
            }

            job.job_id = cupsCreateJob(
                ptr::null_mut::<http_t>(),
                job.pr_name.as_ptr().cast(),
                job.doc_name.as_ptr().cast(),
                num_options,
                options,
            );
            if job.job_id == 0 {
                return Err(PrintError::last_io_error());
            }

            let format = match param.data_format{
                Format::Raw => CUPS_FORMAT_RAW.as_ptr(),
                Format::Auto => CUPS_FORMAT_AUTO.as_ptr(),
                Format::Command => CUPS_FORMAT_COMMAND.as_ptr(),
                Format::Jpeg => CUPS_FORMAT_JPEG.as_ptr(),
                Format::Pdf => CUPS_FORMAT_PDF.as_ptr(),
                Format::Postscript => CUPS_FORMAT_POSTSCRIPT.as_ptr(),
                Format::Text => CUPS_FORMAT_TEXT.as_ptr(),
            };

            if cupsStartDocument(
                ptr::null_mut::<http_t>(),
                job.pr_name.as_ptr().cast(),
                job.job_id,
                job.doc_name.as_ptr().cast(),
                format as *const c_char,
                1,
            ) != HTTP_STATUS_CONTINUE
            {
                Err(PrintError::last_io_error())
            } else {
                Ok(job)
            }
        }
    }

    /// Close the printjob.
    pub fn close(&mut self) -> Result<(), std::io::Error> {
        unsafe {
            if cupsFinishDocument(ptr::null_mut::<http_t>(), self.pr_name.as_ptr().cast())
                == IPP_STATUS_OK
            {
                Ok(())
            } else {
                Err(PrintError::last_io_error())
            }
        }
    }

    /// Start a new page. More a hint to the spooling system, wherever it
    /// displays a page count.
    pub fn start_page(&self) -> Result<(), std::io::Error> {
        Ok(())
    }

    /// End a page.
    pub fn end_page(&self) -> Result<(), std::io::Error> {
        Ok(())
    }
}
