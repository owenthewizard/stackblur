//! Blurs all JPEG and PNG images in directory `input` and places them in directory `output`.

use std::ffi::OsStr;
use std::num::{NonZeroU8, NonZeroUsize};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};

use stackblur::blur;

fn main() -> Result<()> {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join("input");
    let output = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join("output");

    for img_path in input
        .read_dir()
        .with_context(|| format!("Failed to read {:?}", input))?
        .filter_map(|file| {
            let file = file.ok()?;
            let name: PathBuf = file.file_name().into();
            if let Some(ext) = name
                .extension()
                .and_then(OsStr::to_str)
                .map(str::to_ascii_lowercase)
            {
                match ext.as_str() {
                    "png" | "jpeg" | "jpg" => Some(file.path()),
                    _ => {
                        eprintln!("Skipping file with unsupported extension: {:?}", name);
                        None
                    }
                }
            } else {
                eprintln!("Skipping file with invalid extension: {:?}", name);
                None
            }
        })
    {
        let img_name = img_path.file_name().unwrap();
        let mut img = image::open(&img_path)
            .with_context(|| format!("Failed to open {:?} as image", img_name))?
            .into_rgba8();
        let width = NonZeroUsize::new(img.width() as usize).unwrap();
        let height = NonZeroUsize::new(img.height() as usize).unwrap();
        let radius = NonZeroU8::new(15).unwrap();
        let buf = unsafe { img.as_flat_samples_mut().samples.align_to_mut::<u32>().1 };
        let now = Instant::now();
        blur(buf, width, height, radius);
        println!("Blurring {:?} took {:#?}", img_name, now.elapsed());
        img.save(output.join(img_name))
            .with_context(|| format!("Failed to save {:?}", img_name))?;
    }
    Ok(())
}
