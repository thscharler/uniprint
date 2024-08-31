use std::io::Write;

use uniprint::*;

#[test]
fn test_list() {
    println!("listing printers");
    let v = list_printers();
    let _ = dbg!(v);
}

#[test]
fn test_attr() {
    println!("listing attr");
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
fn test_print3() -> std::io::Result<()> {
    let mut param = JobParam::default();
    param.data_format = Format::Text;
    param.copies = Some(2);
    param.orientation = Some(Orientation::Landscape);
    param.duplex = Some(Duplex::Simplex);

    let mut pj = PrintJob::new_with("Olivetti PG L2150 KX", "Document 3", &param)?;

    pj.start_page()?;
    pj.write(b"test2223\n")?;
    pj.close()?;

    Ok(())
}

#[test]
fn test_print2() -> std::io::Result<()> {
    let mut param = JobParam::default();
    param.data_format = Format::Text;
    param.copies = Some(2);
    param.orientation = Some(Orientation::Landscape);
    param.duplex = Some(Duplex::Simplex);

    let mut pj = PrintJob::new_with("Olivetti PG L2150 KX", "Document 2", &param)?;

    pj.start_page()?;
    pj.write(b"test2223\n")?;
    pj.close()?;

    Ok(())
}

#[test]
fn test_print() -> Result<(), std::io::Error> {
    println!("open");
    // let mut pj = PrintJob::new("Olivetti PG L2150", "Document 1")?;
    let mut pj = PrintJob::new("Olivetti PG L2150 KX", "Document 1")?;
    println!("start");
    pj.start_page()?;
    println!("write");
    pj.write(b"test0")?;
    println!("close");
    pj.close()?;

    Ok(())
}
