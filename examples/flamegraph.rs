use stackblur::blur;

use std::num::{NonZeroUsize, NonZeroU8};

const RAILWAY: &'static [u8; 32662000] = include_bytes!("railway.rgba");

fn main() {
    let mut pixels = RAILWAY.to_vec();
    let pixels_32 = unsafe { pixels.align_to_mut::<u32>().1 };
    blur(pixels_32, NonZeroUsize::new(3500).unwrap(), NonZeroUsize::new(2333).unwrap(), NonZeroU8::new(50).unwrap());
}
