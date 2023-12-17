//!
//!
//! References:
//! https://www.rfc-editor.org/rfc/rfc8011
//! https://www.cups.org
//! https://stackoverflow.com/questions/44687154/more-complete-list-of-cups-printer-state-reasons

use std::ffi::{c_int, CStr, CString};
use std::io;
use std::io::{ErrorKind, Write};
use std::ptr::{self, slice_from_raw_parts};

use cups_sys::ippGetInteger;
use cups_sys::{
    cupsCheckDestSupported, cupsCopyDestInfo, cupsCreateJob, cupsFindDestReady, cupsFinishDocument,
    cupsFreeDestInfo, cupsFreeDests, cupsGetDests, cupsGetNamedDest, cupsLastErrorString,
    cupsStartDocument, cupsWriteRequestData, cups_dest_t, cups_option_t,
};
use cups_sys::{
    http_status_e_HTTP_STATUS_CONTINUE as HTTP_STATUS_CONTINUE, http_t,
    ipp_status_e_IPP_STATUS_OK as IPP_STATUS_OK, CUPS_COPIES, CUPS_FORMAT_RAW,
};

use crate::PrintError;

impl PrintError {
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

        Err(io::Error::new(
            ErrorKind::Other,
            PrintError::NoDefaultPrinter,
        ))
    }
}

// printer-state
//      "3" if the destination is idle,
//      "4" if the destination is printing a job,
//      "5" if the destination is stopped.
//
// printer-state-reasons from https://stackoverflow.com/questions/44687154/more-complete-list-of-cups-printer-state-reasons
//
// none
// other
// developer-low
// door-open
// fuser-over-temp
// fuser-under-temp
// input-tray-missing
// interlock-open
// interpreter-resource-unavailable
// marker-supply-empty
// marker-supply-low
// marker-waste-almost-full
// marker-waste-full
// media-empty
// media-jam
// media-low
// media-needed
// moving-to-paused
// opc-life-over
// opc-near-eol
// output-area-almost-full
// output-area-full
// output-tray-missing
// paused
// shutdown
// spool-area-full
// stopped-partly
// stopping
// timed-out
// toner-empty
// toner-low
//
// However the source code of job.c seems to also mention the following statuses (including the two mention in the question). I think this makes an exhaustive list until more come along.
//
// connecting-to-device
// offline-report
// cups-insecure-filter-warning
// cups-missing-filter-warning
// cups-remote-aborted
// cups-remote-canceled
// cups-remote-completed
// cups-remote-pending
// cups-remote-pending-held
// cups-remote-processing
// cups-remote-stopped
// cups-waiting-for-job-completed

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
    pub marker_change_time: u32,

    pub status_none: bool,
    pub status_other: bool,
    pub status_developer_low: bool,
    pub status_door_open: bool,
    pub status_fuser_over_temp: bool,
    pub status_fuser_under_temp: bool,
    pub status_input_tray_missing: bool,
    pub status_interlock_open: bool,
    pub status_interpreter_resource_unavailable: bool,
    pub status_marker_supply_empty: bool,
    pub status_marker_supply_low: bool,
    pub status_waste_almost_full: bool,
    pub status_waste_full: bool,
    pub status_media_empty: bool,
    pub status_media_jam: bool,
    pub status_media_low: bool,
    pub status_media_needed: bool,
    pub status_moving_to_paused: bool,
    pub status_opc_life_over: bool,
    pub status_opc_near_eol: bool,
    pub status_output_area_almost_full: bool,
    pub status_output_area_full: bool,
    pub status_output_tray_missing: bool,
    pub status_paused: bool,
    pub status_shutdown: bool,
    pub status_spool_area_full: bool,
    pub status_stopped_partly: bool,
    pub status_stopping: bool,
    pub status_timed_out: bool,
    pub status_toner_empty: bool,
    pub status_toner_low: bool,
    pub status_connection_to_device: bool,
    pub status_offline_report: bool,
    pub status_insecure_filter_warning: bool,
    pub status_missing_filter_warning: bool,
    pub status_remote_aborted: bool,
    pub status_remote_canceled: bool,
    pub status_remote_completed: bool,
    pub status_remote_pending: bool,
    pub status_remote_pending_held: bool,
    pub status_remote_processing: bool,
    pub status_remote_stopped: bool,
    pub status_waiting_for_job_completed: bool,
}

pub fn printer_attr(pr_name: &str) {
    unsafe {
        let cups_dest = cupsGetNamedDest(ptr::null::<http_t>(), pr_name.as_ptr(), ptr::null());
        if !cups_dest.is_null() {
            let cups_dest = &*cups_dest;

            let result = Info::default();

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

            let opt = cupsGetOption(
                "printer-uri-supported",
                cups_dest.num_options,
                cups_dest.options,
            );

            let c_options = &*slice_from_raw_parts(cur_dest.options, cur_dest.num_options as usize);

            // const char *cupsGetOption(const char *name, int num_options, cups_option_t *options);

            for opt in c_options {
                let c_opt_name = CStr::from_ptr(opt.name);
                let c_opt_value = CStr::from_ptr(opt.value);
                println!("{:?}={:?}", c_opt_name, c_opt_value);
            }

            // Starting with CUPS 1.2, the returned list of destinations
            // include the "printer-info", "printer-is-accepting-jobs",
            // "printer-is-shared", "printer-make-and-model", "printer-state",
            // "printer-state-change-time", "printer-state-reasons",
            // "printer-type", and "printer-uri-supported" attributes as options.
            //
            //     CUPS 1.4 adds the "marker-change-time", "marker-colors",
            // "marker-high-levels", "marker-levels", "marker-low-levels",
            // "marker-message", "marker-names", "marker-types", and
            // "printer-commands" attributes as options.
            //
            //     CUPS 2.2 adds accessible IPP printers to the list of
            // destinations that can be used. The "printer-uri-supported"
            // option will be present for those IPP printers that have been
            // recently used.

            let dinfo = cupsCopyDestInfo(ptr::null_mut::<http_t>(), cur_dest as *mut cups_dest_t);

            let copies = cupsCheckDestSupported(
                ptr::null_mut::<http_t>(),
                cur_dest as *mut cups_dest_t,
                dinfo,
                // CString::new("copies").expect("copies").as_bytes_with_nul().as_ptr() as *const i8,
                CUPS_COPIES.as_ptr() as *const i8,
                ptr::null(),
            ) != 0;
            println!("copies={}", copies);

            let copies = cupsFindDestReady(
                ptr::null_mut::<http_t>(),
                cur_dest as *mut cups_dest_t,
                dinfo,
                CUPS_COPIES.as_ptr() as *const i8,
            );
            let copies = ippGetInteger(copies, 0);
            println!("copies={}", copies);

            cupsFreeDestInfo(dinfo);
        }

        cupsFreeDests(n_dests, cups_dest);
    }
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
                Err(io::Error::new(ErrorKind::Other, PrintError::last_error()))
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
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    PrintError::last_error(),
                ));
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
                Err(io::Error::new(ErrorKind::Other, PrintError::last_error()))
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
                Err(io::Error::new(ErrorKind::Other, PrintError::last_error()))
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
