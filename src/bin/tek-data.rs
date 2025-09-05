use std::ffi::CString;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use visa_rs::{
    enums::attribute::{Attribute, HasAttribute},
    prelude::{AccessMode, AsResourceManager, DefaultRM, Instrument, TIMEOUT_INFINITE},
};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    port: CString,

    #[arg(short, long)]
    channels: Vec<String>,

    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let rm: DefaultRM = DefaultRM::new().unwrap();

    let expr = cli.port.into();
    let rsc = rm.find_res(&expr).unwrap();

    let mut instr: Instrument = rm
        .open(&rsc, AccessMode::NO_LOCK, TIMEOUT_INFINITE)
        .unwrap();

    instr
        .set_attr(Attribute::AttrTmoValue(
            visa_rs::enums::attribute::AttrTmoValue::new_checked(10000).unwrap(),
        ))
        .unwrap();

    let result = tek_utils::query_str(&mut instr, "*IDN?").unwrap();

    print!("Instrument: {}", result);

    tek_utils::write(
        &mut instr,
        &format!("DATA:SOURCE {}", cli.channels.join(",")),
    )
    .unwrap();
    tek_utils::write(&mut instr, "DATA:ENCDG ASCII").unwrap();
    tek_utils::write(&mut instr, "DATA:START").unwrap();

    let bin = tek_utils::query_bin(&mut instr, "WAVFrm?").unwrap();

    let mut file = std::fs::File::create(&cli.output).unwrap();
    file.write_all(&bin).unwrap();

    println!("Saved into {}", cli.output.as_os_str().to_string_lossy());
}
