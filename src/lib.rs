use std::io::{Read, Write};

use visa_rs::Instrument;

pub fn write(mut instr: &Instrument, query: &str) -> std::io::Result<()> {
    println!("> {query}");
    instr.write_all(query.as_bytes())?;
    Ok(())
}

pub fn query_bin(mut instr: &Instrument, query: &str) -> std::io::Result<Vec<u8>> {
    write(instr, query)?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    let mut buf = vec![0; 1024 * 1024];
    let len = instr.read(&mut buf)?;
    buf.truncate(len);

    Ok(buf)
}

pub fn query_str(instr: &Instrument, query: &str) -> std::io::Result<String> {
    let buf = query_bin(instr, query)?;
    let buf = String::from_utf8(buf).unwrap();

    Ok(buf)
}
