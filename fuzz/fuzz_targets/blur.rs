#![no_main]

use libfuzzer_sys::fuzz_target;

use std::num::{NonZeroU8, NonZeroUsize};

use stackblur::blur;

fuzz_target!(|data: &[u8]| {
    let mut v = data.to_vec();
    let pixels = unsafe { v.align_to_mut::<u32>().1 };
    let w = NonZeroUsize::new(data.len() / 640).unwrap();
    let h = NonZeroUsize::new(640).unwrap();
    let r = NonZeroU8::new(15).unwrap();
    blur(pixels, w, h, r);
    println!("{:#?}", pixels);
});
