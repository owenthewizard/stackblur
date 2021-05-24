use std::num::{NonZeroU8, NonZeroUsize};

mod blurstack;
use blurstack::BlurStack;

mod columns;
use columns::*;

const fn pixel(a: u32, r: u32, g: u32, b: u32) -> u32 {
    a << 24 | r << 16 | g << 8 | b
}

/// Performs a pass of stackblur in both directions.
/// Input is expected to be in linear RGB color space.
pub fn blur(src: &mut [u32], width: NonZeroUsize, height: NonZeroUsize, radius: NonZeroU8) {
    blur_horiz(src, width, radius);
    //blur_vert(src, width, height, radius);
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
        let last = *row.last().unwrap();

        let mut queue = BlurStack::with_capacity(2 * r + 1);

        // fill with left edge pixel
        for _ in 0..=r {
            queue.push_back(first);
        }

        // fill with starting pixels
        for i in r + 1..=2 * r {
            queue.push_back(*row.get(i).unwrap_or(&last));
        }

        debug_assert_eq!(queue.len(), 2 * r + 1);

        for (i, r) in (0..width).zip(r..) {
            // set pixel
            let (alpha, red, green, blue) = queue.sums();
            *row.get_mut(i).unwrap() = pixel(alpha / div, red / div, green / div, blue / div);

            // drop left edge of kernel
            let _ = queue.pop_front();

            // add right edge of kernel
            let next = *row.get(r).unwrap_or(&last);
            queue.push_back(next);
        }
    });
}

/*
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
*/

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
