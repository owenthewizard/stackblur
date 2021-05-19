use std::cmp::min;
use std::collections::VecDeque;
use std::iter;
use std::num::{NonZeroU8, NonZeroUsize};

use itertools::peek_nth;

mod blurstack;
use blurstack::BlurStack;

mod columns;
use columns::*;

const MUL_TABLE: [u32; 255] = [
    512, 512, 456, 512, 328, 456, 335, 512, 405, 328, 271, 456, 388, 335, 292, 512, 454, 405, 364,
    328, 298, 271, 496, 456, 420, 388, 360, 335, 312, 292, 273, 512, 482, 454, 428, 405, 383, 364,
    345, 328, 312, 298, 284, 271, 259, 496, 475, 456, 437, 420, 404, 388, 374, 360, 347, 335, 323,
    312, 302, 292, 282, 273, 265, 512, 497, 482, 468, 454, 441, 428, 417, 405, 394, 383, 373, 364,
    354, 345, 337, 328, 320, 312, 305, 298, 291, 284, 278, 271, 265, 259, 507, 496, 485, 475, 465,
    456, 446, 437, 428, 420, 412, 404, 396, 388, 381, 374, 367, 360, 354, 347, 341, 335, 329, 323,
    318, 312, 307, 302, 297, 292, 287, 282, 278, 273, 269, 265, 261, 512, 505, 497, 489, 482, 475,
    468, 461, 454, 447, 441, 435, 428, 422, 417, 411, 405, 399, 394, 389, 383, 378, 373, 368, 364,
    359, 354, 350, 345, 341, 337, 332, 328, 324, 320, 316, 312, 309, 305, 301, 298, 294, 291, 287,
    284, 281, 278, 274, 271, 268, 265, 262, 259, 257, 507, 501, 496, 491, 485, 480, 475, 470, 465,
    460, 456, 451, 446, 442, 437, 433, 428, 424, 420, 416, 412, 408, 404, 400, 396, 392, 388, 385,
    381, 377, 374, 370, 367, 363, 360, 357, 354, 350, 347, 344, 341, 338, 335, 332, 329, 326, 323,
    320, 318, 315, 312, 310, 307, 304, 302, 299, 297, 294, 292, 289, 287, 285, 282, 280, 278, 275,
    273, 271, 269, 267, 265, 263, 261, 259,
];

const SHR_TABLE: [u8; 255] = [
    9, 11, 12, 13, 13, 14, 14, 15, 15, 15, 15, 16, 16, 16, 16, 17, 17, 17, 17, 17, 17, 17, 18, 18,
    18, 18, 18, 18, 18, 18, 18, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 20, 20, 20,
    20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 21, 21, 21, 21, 21, 21, 21, 21, 21,
    21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 22, 22, 22, 22, 22, 22,
    22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
    22, 22, 22, 22, 22, 22, 22, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23,
    23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23,
    23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 23, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
    24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24,
];

const fn red(p: u32) -> u32 {
    (p >> 16) & 0xff
}

const fn green(p: u32) -> u32 {
    (p >> 8) & 0xff
}

const fn blue(p: u32) -> u32 {
    p & 0xff
}

const fn pixel(r: u32, g: u32, b: u32) -> u32 {
    (0xff << 24) | (r << 16) | (g << 8) | b
}

/// Performs a pass of stackblur in both directions.
/// Input is expected to be in linear RGB color space.
pub fn blur(src: &mut [u32], width: NonZeroUsize, height: NonZeroUsize, radius: NonZeroU8) {
    //blur_horiz(src, width, radius);
    blur_vert(src, width, height, radius);
}

/// Performs a horizontal pass of stackblur.
/// Input is expected to be in linear RGB color space.
pub fn blur_horiz(src: &mut [u32], width: NonZeroUsize, radius: NonZeroU8) {
    let width = width.get();
    let radius = u32::from(radius.get());
    let r = radius as usize;
    let div = radius * (radius + 2) + 1;

    src.chunks_exact_mut(width).for_each(|row| {
        let first = *row.first().unwrap();
        let mut last = *row.last().unwrap();

        let mut queue = BlurStack::with_capacity(2 * r + 1);

        // fill with left edge pixel
        for _ in 0..=r {
            queue.push_back(first);
        }

        // fill with starting pixels
        for v in row.iter().copied().chain(iter::repeat(last)).take(r) {
            queue.push_back(v);
        }

        debug_assert_eq!(queue.len(), 2 * r + 1);

        let mut row_iter = peek_nth(row.iter_mut());

        while let Some(px) = row_iter.next() {
            // set pixel
            //
            // using MUL_TABLE and SHR_TABLE didn't speed things up in my testing of 100 iterations
            *px = pixel(
                queue.sum_r() / div,
                queue.sum_g() / div,
                queue.sum_b() / div,
            );

            // drop left edge of kernel
            let _ = queue.pop_front();

            // add right edge of kernel
            let next = **row_iter.peek_nth(r).unwrap_or(&&mut last);
            queue.push_back(next);
        }
    });
}

/// Performs a vertical pass of stackblur.
/// Input is expected to be in linear RGB color space.
pub fn blur_vert(src: &mut [u32], width: NonZeroUsize, height: NonZeroUsize, radius: NonZeroU8) {
    let width = width.get();
    let height = height.get();
    let radius = u32::from(radius.get());
    let r = radius as usize;
    let div = radius * (radius + 2) + 1;

    for x in 0..width {
        let first = src[x];
        let mut last = src[x + width * (height - 1)];

        // backfill queue with starting pixels
        let mut queue = src
            .column(x, width)
            .copied()
            .chain(iter::repeat(last))
            .take(r)
            .collect::<VecDeque<u32>>();
        queue.reserve_exact(2 * r + 1);
        let mut queue = BlurStack::from(queue);

        let mut col_iter = peek_nth(src.column_mut(x, width));

        // fill with top edge pixel
        for _ in 0..=r {
            queue.push_front(first);
        }

        debug_assert_eq!(queue.len(), 2 * r + 1);

        while let Some(px) = col_iter.next() {
            // set pixel
            //
            // using MUL_TABLE and SHR_TABLE didn't speed things up in my testing of 100 iterations
            *px = pixel(
                queue.sum_r() / div,
                queue.sum_g() / div,
                queue.sum_b() / div,
            );

            // drop top edge of kernel
            let _ = queue.pop_front();

            // add bottom edge of kernel
            let next = **col_iter.peek_nth(r).unwrap_or(&&mut last);
            queue.push_back(next);
        }
    }
}

#[cfg(test)]
mod test {
    use std::num::{NonZeroU8, NonZeroUsize};

    use super::blur;

    #[test]
    fn tiny_image() {
        let mut v = vec![0x12345678];
        let w = NonZeroUsize::new(1).unwrap();
        let h = NonZeroUsize::new(1).unwrap();
        let r = NonZeroU8::new(1).unwrap();

        blur(&mut v, w, h, r);

        dbg!(v);
    }

    #[test]
    fn tiny_image_large_radius() {
        let mut v = vec![0x12345678; 9];
        let w = NonZeroUsize::new(3).unwrap();
        let h = NonZeroUsize::new(3).unwrap();
        let r = NonZeroU8::new(u8::MAX).unwrap();

        blur(&mut v, w, h, r);

        dbg!(v);
    }
}
