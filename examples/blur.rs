//! Blurs the sRGB RGBA PNG supplied at argv[1] by a radius supplied at argv[2].
//!
//! # Example
//!
//! ```shell
//! # blur cballs.png by a radius of 5
//! blur cballs.png 15
//! ```

use std::env;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{BufWriter, Seek, SeekFrom};
use std::num::{NonZeroU8, NonZeroUsize};
use std::path::PathBuf;

use anyhow::{anyhow, ensure, Context, Result};
use png::{ColorType, Decoder, Encoder};

use stackblur::blur;

/// Converts gamma-encoded sRGB color to linear sRGB.
fn srgb_to_linear(color: u8) -> u8 {
    let color = f32::from(color) / 255.0;
    if color <= 0.04045 {
        ((color / 12.92) * 255.0) as u8
    } else {
        (((color + 0.055) / 1.055).powf(2.4) * 255.0) as u8
    }
}

/// Converts linear sRGB color to gamma-encoded sRGB.
fn linear_to_srgb(color: u8) -> u8 {
    let color = f32::from(color) / 255.0;
    if color <= 0.0031308 {
        ((color * 12.92) * 255.0) as u8
    } else {
        ((color.powf(1.0 / 2.4) * 1.055 - 0.055) * 255.0) as u8
    }
}

fn usage() -> anyhow::Error {
    anyhow!("usage: blur <picture.png> <radius>")
}

fn main() -> Result<()> {
    let args = env::args_os().skip(1).take(2).collect::<Vec<OsString>>();
    if let Some(_) = args.get(2) {
        return Err(usage());
    }

    let png = PathBuf::from(args.get(0).ok_or_else(usage)?);

    let radius = args
        .get(1)
        .ok_or_else(usage)?
        .to_str()
        .ok_or_else(usage)?
        .parse::<NonZeroU8>()
        .context("Radius must be [1-255]")?;

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(png)
        .context("Failed to open PNG for reading and writing")?;

    let decoder = Decoder::new(&file);
    let (info, mut reader) = decoder.read_info().context("Failed to read PNG info")?;

    ensure!(info.color_type == ColorType::RGBA, "PNG must be RGBA");

    let mut buffer = vec![0; info.buffer_size()];
    reader
        .next_frame(&mut buffer)
        .context("Failed to decode PNG")?;

    buffer.iter_mut().for_each(|c| *c = srgb_to_linear(*c));

    unsafe {
        blur(
            buffer.align_to_mut::<u32>().1,
            NonZeroUsize::new(info.width as usize).unwrap(),
            NonZeroUsize::new(info.height as usize).unwrap(),
            radius,
        );
    }

    buffer.iter_mut().for_each(|c| *c = linear_to_srgb(*c));

    let w = &mut BufWriter::new(file);
    let _ = w
        .seek(SeekFrom::Start(0))
        .context("Failed to seek to start of file")?;

    let mut encoder = Encoder::new(w, info.width, info.height);
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .context("Failed to write PNG header")?;
    writer
        .write_image_data(&buffer)
        .context("Failed to write PNG data")?;

    Ok(())
}
