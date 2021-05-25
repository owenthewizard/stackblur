//! Fast gaussian blur approximation.
//!
//! Rust implementation of the [StackBlur algorithm](https://medium.com/mobile-app-development-publication/blurring-image-algorithm-example-in-android-cec81911cd5e#bfcb)
//! by [Mario Klingemann](https://underdestruction.com/2004/02/25/stackblur-2004).
//! Based off of the [Java implementation](https://github.com/kikoso/android-stackblur)
//! by Enrique López Mañas, licensed under Apache 2.0.
//!
//! # Examples
//!
//! ```rust no_run
//! use std::num::{NonZeroU8, NonZeroUsize};
//!
//! use stackblur::blur;
//!
//! // load your image, u32 RGBA pixels
//! let mut pixels: Vec<u32> = unimplemented!();
//!
//! // blur!
//! blur(
//!     &mut pixels,
//!     NonZeroUsize::new(1024).unwrap(),
//!     NonZeroUsize::new(768).unwrap(),
//!     NonZeroU8::new(15).unwrap(),
//! );
//!
//! ```

mod blur;
pub use blur::blur;
pub use blur::blur_horiz;
//pub use blur::blur_vert;
