[![crates.io](https://img.shields.io/crates/v/uniprint.svg)](https://crates.io/crates/uniprint)
[![Documentation](https://docs.rs/uniprint/badge.svg)](https://docs.rs/uniprint)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/uniprint)

uniprint
====

Tries to give a unified view over Windows and CUPS printing.

It can now

- list the installed printers.
- list the default printer.
- request the printer state and settings.
- start a print-job with job-parameters and send data.

Limitations
====

Printer state and settings are impossible to unify between these two worlds.
This library tries to give a rough estimate of the state, and allows access to
all system specific flags.

The same with parameters for a print-job. There are diverging capabilities in
both parameters and allowable values for parameters. But all documented
parameters are available for the respective systems.

Third, the actual print data that can be sent to the printer is also different.
The only common options are text and raw formats.


Current usage
====

Currently, I use this to print labels. This uses the native printer language
and sends the raw data format to the printer. Using these escape
sequences uses a lot less data than sending images, and the processing in
the printer is faster too.








