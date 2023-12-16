use std::ffi::{c_int, CStr, CString};
use std::io;
use std::io::{ErrorKind, Write};
use std::ptr::{self, slice_from_raw_parts};

use cups_sys::ippGetInteger;
use cups_sys::{
    cupsCheckDestSupported, cupsCopyDestInfo, cupsCreateJob, cupsFindDestReady, cupsFinishDocument,
    cupsFreeDestInfo, cupsFreeDests, cupsGetDests, cupsLastErrorString, cupsStartDocument,
    cupsWriteRequestData, cups_dest_t, cups_option_t,
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

pub fn printer_attr(pr_name: &str) {
    unsafe {
        let mut cups_dest: *mut cups_dest_t = ptr::null_mut::<cups_dest_t>();
        let pcups_dest = (&mut cups_dest) as *mut *mut cups_dest_t;

        let n_dests = cupsGetDests(pcups_dest);

        for i in 0isize..n_dests as isize {
            let cur_dest = &mut *cups_dest.offset(i);

            dbg!("21");

            let c_name = CStr::from_ptr(cur_dest.name);
            let c_instance = if !cur_dest.instance.is_null() {
                Some(CStr::from_ptr(cur_dest.instance))
            } else {
                None
            };
            println!("{:?} {:?}", c_name, c_instance);

            let c_options = &*slice_from_raw_parts(cur_dest.options, cur_dest.num_options as usize);
            for opt in c_options {
                let c_opt_name = CStr::from_ptr(opt.name);
                let c_opt_value = CStr::from_ptr(opt.value);
                println!("{:?}={:?}", c_opt_name, c_opt_value);
            }

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
            println!("copies={}",copies);

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
