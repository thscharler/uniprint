//!
//!
//! References:
//! https://www.rfc-editor.org/rfc/rfc8011
//! https://www.cups.org
//! https://stackoverflow.com/questions/44687154/more-complete-list-of-cups-printer-state-reasons

use std::ffi::{c_int, CStr, CString};
use std::io::{ErrorKind, Write};
use std::ptr::{self, slice_from_raw_parts};
use std::str::FromStr;

use cups_sys::{
    cupsCreateJob, cupsFinishDocument, cupsFreeDests, cupsGetDests, 
    cupsGetNamedDest, cupsLastErrorString, cupsStartDocument, cupsWriteRequestData,
};
use cups_sys::{cups_dest_t, cups_option_t};
use cups_sys::{
    http_status_e_HTTP_STATUS_CONTINUE as HTTP_STATUS_CONTINUE, http_t,
    ipp_status_e_IPP_STATUS_OK as IPP_STATUS_OK, CUPS_FORMAT_RAW,
};

use crate::PrintError;

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

#[derive(Default, Debug, Clone)]
pub enum ColorMode {
    #[default]
    Auto,
    Monochrome,
    Color,
}

#[derive(Default, Debug, Clone)]
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

#[derive(Default, Debug, Clone)]
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
            pr_name.as_ptr() as *const i8,
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

            let opt = find_option(
                "print-color-mode",
                cups_dest.num_options,
                cups_dest.options,
            );
            result.print_color_mode = option_map(
                &opt,
                &[
                    ("auto", ColorMode::Auto),
                    ("monochrome", ColorMode::Monochrome),
                    ("color", ColorMode::Color),
                ],
            );

            result.copies = find_num_option(
                "copies",
                cups_dest.num_options,
                cups_dest.options,
            )?;

            let opt = find_option(
                "finishings",
                cups_dest.num_options,
                cups_dest.options,
            );
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

            result.number_up = find_num_option(
                "number-up",
                cups_dest.num_options,
                cups_dest.options,
            )?;

            let opt = find_option(
                "printer-is-accepting-jobs",
                cups_dest.num_options,
                cups_dest.options,
            );
            result.printer_is_accepting_jobs = option_map(&opt, &[("true", true), ("false", false)]);

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

            let opt = find_option(
                "printer-state",
                cups_dest.num_options,
                cups_dest.options,
            );
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

            result.printer_type = find_num_option(
                "printer-type",
                cups_dest.num_options,
                cups_dest.options,
            )?;

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
//                // CString::new("copies").expect("copies").as_bytes_with_nul().as_ptr() as *const i8,
//                CUPS_COPIES.as_ptr() as *const i8,
//                ptr::null(),
//            ) != 0;
//            println!("copies={}", copies);
//
//            let copies = cupsFindDestReady(
//                ptr::null_mut::<http_t>(),
//                cups_dest as *mut cups_dest_t,
//                dinfo,
//                CUPS_COPIES.as_ptr() as *const i8,
//            );
//            let copies = ippGetInteger(copies, 0);
//            println!("copies={}", copies);
//
//            cupsFreeDestInfo(dinfo);

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
                buf.as_ptr() as *const i8,
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
    pub fn new(pr_name: &str, doc_name: &str) -> Result<Self, std::io::Error> {
        let mut job = LinuxPrintJob {
            pr_name: CString::new(pr_name)?,
            doc_name: CString::new(doc_name)?,
            job_id: 0,
        };

        unsafe {
            job.job_id = cupsCreateJob(
                ptr::null_mut::<http_t>(),
                job.pr_name.as_ptr().cast(),
                job.doc_name.as_ptr().cast(),
                0,
                ptr::null_mut::<cups_option_t>(),
            );
            if job.job_id == 0 {
                return Err(PrintError::last_io_error());
            }

            if cupsStartDocument(
                ptr::null_mut::<http_t>(),
                job.pr_name.as_ptr().cast(),
                job.job_id,
                job.doc_name.as_ptr().cast(),
                CUPS_FORMAT_RAW.as_ptr().cast(),
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
