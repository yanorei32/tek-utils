use std::ffi::CString;
use std::path::PathBuf;

use clap::Parser;
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use image::{DynamicImage, ImageBuffer, Rgb};
use tinybmp::{Bmp, RawBmp};
use visa_rs::{
    enums::attribute::{Attribute, HasAttribute},
    prelude::{AccessMode, AsResourceManager, DefaultRM, Instrument, TIMEOUT_INFINITE},
};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    port: CString,

    #[arg(short, long)]
    output: PathBuf,
}

fn parse_broken_bmp(bin: &[u8]) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let mut bin = bin.to_vec();

    // Set fake biSizeImage
    bin[0x22] = 0;
    bin[0x23] = 0;
    bin[0x24] = 0;
    bin[0x25] = 0;

    // Get actual header size
    let header_length = RawBmp::from_slice(&bin).unwrap().header().image_data_start;

    // Calculate and writeback actual biSizeImage
    let bi_size_image = bin.len() - header_length;
    let bi_size_image = (bi_size_image as u32).to_le_bytes();

    // Write-back actual biSizeImage
    bin[0x22] = bi_size_image[0];
    bin[0x23] = bi_size_image[1];
    bin[0x24] = bi_size_image[2];
    bin[0x25] = bi_size_image[3];

    let bmp: Bmp<'_, Rgb888> = Bmp::from_slice(&bin).unwrap();

    let rgb_buffer = bmp
        .pixels()
        .into_iter()
        .map(|pixel| [pixel.1.r(), pixel.1.g(), pixel.1.b()])
        .flatten()
        .collect::<Vec<_>>();

    let image =
        ImageBuffer::<Rgb<u8>, _>::from_raw(bmp.size().width, bmp.size().height, rgb_buffer)
            .unwrap();

    Ok(DynamicImage::from(image).flipv())
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

    tek_utils::write(&mut instr, "HARDCOPY:LAYOUT PORTRAIT").unwrap();
    tek_utils::write(&mut instr, "HARDCOPY:PORT GPIB").unwrap();
    tek_utils::write(&mut instr, "HARDCOPY:FORMAT RLE").unwrap();

    let bin = tek_utils::query_bin(&mut instr, "HARDCOPY START").unwrap();

    parse_broken_bmp(&bin).unwrap().save(&cli.output).unwrap();

    println!("Saved into {}", cli.output.as_os_str().to_string_lossy());
}
