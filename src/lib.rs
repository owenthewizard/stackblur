//! Fast gaussian blur approximation.
//!
//! A Rust implementation of [`StackBlur`](https://github.com/flozz/StackBlur) by [Mario
//! Klingemann](https://underdestruction.com).
//! Very fast and accurate gaussian blur approximation.
//! Based off of [Java implementation](https://github.com/verzqli/QQBlurView/blob/master/blurview/src/main/java/com/verzqli/blurview/stackblur/JavaBlurProcess.java) by Enrique López Mañas, licensed under Apache 2.0.
//!
//! # Examples
//!
//! ```rust
//! use std::num::{NonZeroU8, NonZeroUsize};
//!
//! use stackblur::blur;
//!
//! const RED: u32 = 0xffff0000;
//! const GREEN: u32 = 0xff00ff00;
//! const BLUE: u32 = 0xff0000ff;
//!
//! // load your image, u32 RGBA pixels
//! let mut pixels: Vec<u32> = vec![
//!     RED, GREEN, GREEN, RED,
//!     GREEN, RED, BLUE, GREEN,
//!     GREEN, BLUE, RED, GREEN,
//!     RED, GREEN, GREEN, RED,
//! ];
//!
//! // blur!
//! blur(
//!     &mut pixels,
//!     NonZeroUsize::new(4).unwrap(),
//!     NonZeroUsize::new(4).unwrap(),
//!     NonZeroU8::new(1).unwrap(),
//! );
//!
//! ```

mod blur;
pub use blur::blur;
pub use blur::blur_horiz;
//pub use blur::blur_vert;
