use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::cmp::min;
use std::ffi::{c_void, OsString};
use std::io::{ErrorKind, Write};
use std::iter::once;
use std::mem::align_of;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr;
use std::ptr::slice_from_raw_parts;

use libc::{wchar_t, wcslen};
use windows_sys::core::{PCWSTR, PWSTR};
use windows_sys::Win32::Foundation::{GetLastError, LocalFree, FALSE, HANDLE, HLOCAL, TRUE};
use windows_sys::Win32::Graphics::Gdi::{
    CCHFORMNAME, DM_COLLATE, DM_COLOR, DM_COPIES, DM_DEFAULTSOURCE, DM_DITHERTYPE, DM_DUPLEX,
    DM_FORMNAME, DM_ICMINTENT, DM_ICMMETHOD, DM_MEDIATYPE, DM_NUP, DM_ORIENTATION, DM_PAPERLENGTH,
    DM_PAPERSIZE, DM_PAPERWIDTH, DM_PRINTQUALITY, DM_SCALE, DM_TTOPTION, DM_YRESOLUTION,
};
use windows_sys::Win32::Graphics::Printing::{
    ClosePrinter, EndDocPrinter, EndPagePrinter, EnumPrintersW, GetDefaultPrinterW, OpenPrinterW,
    StartDocPrinterW, StartPagePrinter, WritePrinter, DOC_INFO_1W, PRINTER_ATTRIBUTE_DEFAULT,
    PRINTER_ATTRIBUTE_DIRECT, PRINTER_ATTRIBUTE_DO_COMPLETE_FIRST, PRINTER_ATTRIBUTE_ENABLE_BIDI,
    PRINTER_ATTRIBUTE_ENABLE_DEVQ, PRINTER_ATTRIBUTE_ENTERPRISE_CLOUD, PRINTER_ATTRIBUTE_FAX,
    PRINTER_ATTRIBUTE_FRIENDLY_NAME, PRINTER_ATTRIBUTE_HIDDEN, PRINTER_ATTRIBUTE_KEEPPRINTEDJOBS,
    PRINTER_ATTRIBUTE_LOCAL, PRINTER_ATTRIBUTE_MACHINE, PRINTER_ATTRIBUTE_NETWORK,
    PRINTER_ATTRIBUTE_PER_USER, PRINTER_ATTRIBUTE_PUBLISHED, PRINTER_ATTRIBUTE_PUSHED_MACHINE,
    PRINTER_ATTRIBUTE_PUSHED_USER, PRINTER_ATTRIBUTE_QUEUED, PRINTER_ATTRIBUTE_RAW_ONLY,
    PRINTER_ATTRIBUTE_SHARED, PRINTER_ATTRIBUTE_TS, PRINTER_ATTRIBUTE_TS_GENERIC_DRIVER,
    PRINTER_ATTRIBUTE_WORK_OFFLINE, PRINTER_ENUM_LOCAL, PRINTER_INFO_2W, PRINTER_INFO_4W,
    PRINTER_STATUS_BUSY, PRINTER_STATUS_DOOR_OPEN, PRINTER_STATUS_ERROR,
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

use crate::PrintError;

extern "C" {
    pub fn wcscmp(__s1: *const wchar_t, __s2: *const wchar_t) -> ::libc::c_int;
    // pub fn wcsncmp(__s1: *const wchar_t, __s2: *const wchar_t, __n: size_t) -> ::libc::c_int;
}

impl PrintError {
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

#[derive(Default, Debug, Clone)]
pub struct Info {
    pub server_name: Option<String>,
    pub port_name: Option<String>,
    pub driver_name: Option<String>,
    pub comment: Option<String>,
    pub location: Option<String>,
    pub sep_file: Option<String>,
    pub print_processor: Option<String>,
    pub data_type: Option<String>,
    pub parameters: Option<String>,

    pub default_priority: u32,
    pub start_time: u32,
    pub until_time: u32,
    pub jobs: u32,
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

/// Extended attributes
pub fn printer_attr(pr_name: &str) -> std::io::Result<Info> {
    let pr_name = OsString::from(pr_name)
        .encode_wide()
        .chain(once(0))
        .collect::<Vec<u16>>();

    unsafe {
        let mut cb_needed = 0u32;
        let mut c_returned = 0u32;

        if EnumPrintersW(
            PRINTER_ENUM_LOCAL,
            ptr::null_mut(),
            2,
            ptr::null_mut(),
            0,
            &mut cb_needed as *mut u32,
            &mut c_returned as *mut u32,
        ) == FALSE
        {
            let info_layout = Layout::from_size_align_unchecked(
                cb_needed as usize,
                align_of::<PRINTER_INFO_2W>(),
            );
            let buf = alloc_zeroed(info_layout);

            if EnumPrintersW(
                PRINTER_ENUM_LOCAL,
                ptr::null_mut(),
                2,
                buf,
                cb_needed,
                &mut cb_needed as *mut u32,
                &mut c_returned as *mut u32,
            ) == TRUE
            {
                let mut result = None;

                for i in 0..c_returned as isize {
                    let info = &*(buf as *mut PRINTER_INFO_2W).offset(i);

                    if 0 == wcscmp(info.pPrinterName, pr_name.as_ptr()) {
                        let mut res = Info::default();

                        res.server_name = extract_wstr(info.pServerName);
                        res.port_name = extract_wstr(info.pPortName);
                        res.driver_name = extract_wstr(info.pDriverName);
                        res.comment = extract_wstr(info.pComment);
                        res.location = extract_wstr(info.pLocation);
                        res.sep_file = extract_wstr(info.pSepFile);
                        res.print_processor = extract_wstr(info.pPrintProcessor);
                        res.data_type = extract_wstr(info.pDatatype);
                        res.parameters = extract_wstr(info.pParameters);
                        res.default_priority = info.DefaultPriority;
                        res.start_time = info.StartTime;
                        res.until_time = info.UntilTime;

                        res.attr_queued = (info.Attributes & PRINTER_ATTRIBUTE_QUEUED) != 0;
                        res.attr_direct = (info.Attributes & PRINTER_ATTRIBUTE_DIRECT) != 0;
                        res.attr_default = (info.Attributes & PRINTER_ATTRIBUTE_DEFAULT) != 0;
                        res.attr_network = (info.Attributes & PRINTER_ATTRIBUTE_NETWORK) != 0;
                        res.attr_shared = (info.Attributes & PRINTER_ATTRIBUTE_SHARED) != 0;
                        res.attr_hidden = (info.Attributes & PRINTER_ATTRIBUTE_HIDDEN) != 0;
                        res.attr_local = (info.Attributes & PRINTER_ATTRIBUTE_LOCAL) != 0;
                        res.attr_enable_devq =
                            (info.Attributes & PRINTER_ATTRIBUTE_ENABLE_DEVQ) != 0;
                        res.attr_keep_printed_jobs =
                            (info.Attributes & PRINTER_ATTRIBUTE_KEEPPRINTEDJOBS) != 0;
                        res.attr_do_complete_first =
                            (info.Attributes & PRINTER_ATTRIBUTE_DO_COMPLETE_FIRST) != 0;
                        res.attr_work_offline =
                            (info.Attributes & PRINTER_ATTRIBUTE_WORK_OFFLINE) != 0;
                        res.attr_enable_bidi =
                            (info.Attributes & PRINTER_ATTRIBUTE_ENABLE_BIDI) != 0;
                        res.attr_raw_only = (info.Attributes & PRINTER_ATTRIBUTE_RAW_ONLY) != 0;
                        res.attr_published = (info.Attributes & PRINTER_ATTRIBUTE_PUBLISHED) != 0;
                        res.attr_fax = (info.Attributes & PRINTER_ATTRIBUTE_FAX) != 0;
                        res.attr_ts = (info.Attributes & PRINTER_ATTRIBUTE_TS) != 0;
                        res.attr_pushed_user =
                            (info.Attributes & PRINTER_ATTRIBUTE_PUSHED_USER) != 0;
                        res.attr_pushed_machine =
                            (info.Attributes & PRINTER_ATTRIBUTE_PUSHED_MACHINE) != 0;
                        res.attr_machine = (info.Attributes & PRINTER_ATTRIBUTE_MACHINE) != 0;
                        res.attr_friendly_name =
                            (info.Attributes & PRINTER_ATTRIBUTE_FRIENDLY_NAME) != 0;
                        res.attr_ts_generic_driver =
                            (info.Attributes & PRINTER_ATTRIBUTE_TS_GENERIC_DRIVER) != 0;
                        res.attr_per_user = (info.Attributes & PRINTER_ATTRIBUTE_PER_USER) != 0;
                        res.attr_enterprise_cloud =
                            (info.Attributes & PRINTER_ATTRIBUTE_ENTERPRISE_CLOUD) != 0;

                        res.status_busy = (info.Status & PRINTER_STATUS_BUSY) != 0;
                        res.status_door_open = (info.Status & PRINTER_STATUS_DOOR_OPEN) != 0;
                        res.status_error = (info.Status & PRINTER_STATUS_ERROR) != 0;
                        res.status_initializing = (info.Status & PRINTER_STATUS_INITIALIZING) != 0;
                        res.status_io_active = (info.Status & PRINTER_STATUS_IO_ACTIVE) != 0;
                        res.status_manual_feed = (info.Status & PRINTER_STATUS_MANUAL_FEED) != 0;
                        res.status_no_toner = (info.Status & PRINTER_STATUS_NO_TONER) != 0;
                        res.status_not_available =
                            (info.Status & PRINTER_STATUS_NOT_AVAILABLE) != 0;
                        res.status_offline = (info.Status & PRINTER_STATUS_OFFLINE) != 0;
                        res.status_out_of_memory =
                            (info.Status & PRINTER_STATUS_OUT_OF_MEMORY) != 0;
                        res.status_output_bin_full =
                            (info.Status & PRINTER_STATUS_OUTPUT_BIN_FULL) != 0;
                        res.status_page_punt = (info.Status & PRINTER_STATUS_PAGE_PUNT) != 0;
                        res.status_paper_jam = (info.Status & PRINTER_STATUS_PAPER_JAM) != 0;
                        res.status_paper_out = (info.Status & PRINTER_STATUS_PAPER_OUT) != 0;
                        res.status_paper_problem =
                            (info.Status & PRINTER_STATUS_PAPER_PROBLEM) != 0;
                        res.status_paused = (info.Status & PRINTER_STATUS_PAUSED) != 0;
                        res.status_pending_deletion =
                            (info.Status & PRINTER_STATUS_PENDING_DELETION) != 0;
                        res.status_power_save = (info.Status & PRINTER_STATUS_POWER_SAVE) != 0;
                        res.status_printing = (info.Status & PRINTER_STATUS_PRINTING) != 0;
                        res.status_processing = (info.Status & PRINTER_STATUS_PROCESSING) != 0;
                        res.status_server_unknown =
                            (info.Status & PRINTER_STATUS_SERVER_UNKNOWN) != 0;
                        res.status_toner_low = (info.Status & PRINTER_STATUS_TONER_LOW) != 0;
                        res.status_user_intervention =
                            (info.Status & PRINTER_STATUS_USER_INTERVENTION) != 0;
                        res.status_waiting = (info.Status & PRINTER_STATUS_WAITING) != 0;
                        res.status_warming_up = (info.Status & PRINTER_STATUS_WARMING_UP) != 0;

                        let dev_mode = &*info.pDevMode;
                        res.device_driver_version = dev_mode.dmDriverVersion;

                        if 0 != dev_mode.dmFields & DM_ORIENTATION {
                            res.device_orientation =
                                Some(dev_mode.Anonymous1.Anonymous1.dmOrientation);
                        }
                        if 0 != dev_mode.dmFields & DM_PAPERSIZE {
                            res.device_paper_size =
                                Some(dev_mode.Anonymous1.Anonymous1.dmPaperSize);
                        }
                        if 0 != dev_mode.dmFields & DM_PAPERLENGTH {
                            res.device_paper_length =
                                Some(dev_mode.Anonymous1.Anonymous1.dmPaperLength);
                        }
                        if 0 != dev_mode.dmFields & DM_PAPERWIDTH {
                            res.device_paper_width =
                                Some(dev_mode.Anonymous1.Anonymous1.dmPaperWidth);
                        }
                        if 0 != dev_mode.dmFields & DM_SCALE {
                            res.device_paper_scale = Some(dev_mode.Anonymous1.Anonymous1.dmScale);
                        }
                        if 0 != dev_mode.dmFields & DM_COPIES {
                            res.device_paper_copies = Some(dev_mode.Anonymous1.Anonymous1.dmCopies);
                        }
                        if 0 != dev_mode.dmFields & DM_DEFAULTSOURCE {
                            res.device_default_source =
                                Some(dev_mode.Anonymous1.Anonymous1.dmDefaultSource);
                        }
                        if 0 != dev_mode.dmFields & DM_PRINTQUALITY {
                            res.device_print_quality =
                                Some(dev_mode.Anonymous1.Anonymous1.dmPrintQuality);
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

                        result = Some(res);
                        break;
                    }
                }

                dealloc(buf, info_layout);

                if let Some(result) = result {
                    return Ok(result);
                } else {
                    return Err(std::io::Error::new(ErrorKind::Other, PrintError::NotFound));
                }
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
        unsafe {
            let pr_name = OsString::from(pr_name)
                .encode_wide()
                .chain(once(0))
                .collect::<Vec<u16>>();

            let mut print = WindowsPrintJob {
                printer: 0,
                job_id: 0,
            };

            if 0 != OpenPrinterW(
                pr_name.as_ptr() as PCWSTR,
                &mut print.printer as *mut HANDLE,
                ptr::null(),
            ) {
                let mut doc_name = OsString::from(doc_name)
                    .encode_wide()
                    .chain(once(0))
                    .collect::<Vec<u16>>();
                let doc_info = DOC_INFO_1W {
                    pDocName: doc_name.as_mut_ptr(),
                    pOutputFile: ptr::null_mut(),
                    pDatatype: ptr::null_mut(),
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

fn extract_wstr(value: PWSTR) -> Option<String> {
    unsafe {
        if !value.is_null() {
            let len = wcslen(value);
            let slice = slice_from_raw_parts(value, len);

            let os_str = OsString::from_wide(&*slice);

            Some(os_str.to_string_lossy().to_string())
        } else {
            None
        }
    }
}
