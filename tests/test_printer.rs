use std::io::Write;

use printing::*;

#[test]
fn test_list() {
    println!("listing printers");
    let v = list_printers();
    let _ = dbg!(v);
}

#[test]
fn test_attr() {
    println!("listing printers");
    if let Ok(v) = list_printers() {
        for p in v {
            let v = printer_attr(p.as_str());
            let _ = dbg!(v);
        }
    } else {
        println!("no printers");
    }
}

#[test]
fn test_default() {
    println!("default printer");
    let v = default_printer();
    let _ = dbg!(v);
}

#[test]
fn test_print() -> Result<(), std::io::Error> {
    println!("open");
    // let mut pj = PrintJob::new("Olivetti PG L2150", "Document 1")?;
    let mut pj = PrintJob::new("PGL2150", "Document 1")?;
    println!("start");
    pj.start_page()?;
    println!("write");
    pj.write(b"test0")?;
    println!("close");
    pj.close()?;

    Ok(())
}
