//! Blurs all sRGB RGBA PNG files in directory `input` and places them in directory `output`.

use std::ffi::OsStr;
use std::fs::File;
use std::io::BufWriter;
use std::num::{NonZeroU8, NonZeroUsize};
use std::path::Path;

use anyhow::{ensure, Context, Result};
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

fn main() -> Result<()> {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join("input");
    let output = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join("output");

    for png in input
        .read_dir()
        .with_context(|| format!("Failed to read {:?}", input))?
    {
        let png = png.with_context(|| format!("Failed to read PNG in {:?}", input))?;
        if let Some("png") = png.path().extension().and_then(OsStr::to_str) {
            println!("Decoding {:?}...", png.file_name());
            let decoder = Decoder::new(File::open((png).path()).context("Failed to open PNG")?);
            let (info, mut reader) = decoder.read_info().context("Failed to read PNG info")?;
            ensure!(
                info.color_type == ColorType::RGBA,
                "Sample image must be RGBA",
            );
            let mut buffer = vec![0; info.buffer_size()];
            reader
                .next_frame(&mut buffer)
                .context("Failed to decode PNG")?;

            println!("Converting from gamma-encoded sRGB to linear...");
            buffer.iter_mut().for_each(|c| *c = srgb_to_linear(*c));

            let buf32 = unsafe { buffer.align_to_mut::<u32>().1 };
            println!("Blurring...");
            blur(
                buf32,
                NonZeroUsize::new(info.width as usize).unwrap(),
                NonZeroUsize::new(info.height as usize).unwrap(),
                NonZeroU8::new(15).unwrap(),
            );

            println!("Converting back to gamma-encoded sRGB...");
            buffer.iter_mut().for_each(|c| *c = linear_to_srgb(*c));

            println!("Saving to output...\n");
            let out = output.join(png.file_name());
            let file = File::create(&out).with_context(|| format!("Failed to create {:?}", out))?;
            let w = &mut BufWriter::new(file);
            let mut encoder = Encoder::new(w, info.width, info.height);
            encoder.set_color(png::ColorType::RGBA);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder
                .write_header()
                .context("Failed to write PNG header")?;
            writer
                .write_image_data(&buffer)
                .context("Failed to write PNG data")?;
        }
    }

    println!("All done!");
    Ok(())
}
