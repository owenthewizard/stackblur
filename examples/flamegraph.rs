//! Barebones blur example used for profiling.

use stackblur::blur;

use std::num::{NonZeroU8, NonZeroUsize};

/// Railway image by [Martin Winkler](https://pixabay.com/users/fotoworkshop4you-2995268/?utm_source=link-attribution&amp;utm_medium=referral&amp;utm_campaign=image&amp;utm_content=1555348") from [Pixabay](https://pixabay.com/?utm_source=link-attribution&amp;utm_medium=referral&amp;utm_campaign=image&amp;utm_content=1555348).
const RAILWAY: &'static [u8; 32662000] = include_bytes!("railway.rgba");

fn main() {
    let mut pixels = RAILWAY.to_vec();
    let pixels_32 = unsafe { pixels.align_to_mut::<u32>().1 };
    let width = NonZeroUsize::new(3500).unwrap();
    let height = NonZeroUsize::new(2333).unwrap();
    let radius = NonZeroU8::new(50).unwrap();
    do_blur(pixels_32, width, height, radius);
}

fn do_blur(buf: &mut [u32], width: NonZeroUsize, height: NonZeroUsize, radius: NonZeroU8) {
    blur(buf, width, height, radius);
}
