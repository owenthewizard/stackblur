use std::collections::VecDeque;
use std::iter;
use std::num::{NonZeroU8, NonZeroUsize};

const fn alpha(p: u32) -> u32 {
    (p >> 24) & 0xff
}

const fn red(p: u32) -> u32 {
    (p >> 16) & 0xff
}

const fn green(p: u32) -> u32 {
    (p >> 8) & 0xff
}

const fn blue(p: u32) -> u32 {
    p & 0xff
}

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

    let mut queue = VecDeque::with_capacity(2 * r + 1);
    src.chunks_exact_mut(width).for_each(|row| {
        let first = *row.first().unwrap();
        let last = *row.last().unwrap();

        queue.clear();

        let (mut sum_a, mut sum_r, mut sum_g, mut sum_b) = (0, 0, 0, 0);
        // fill with left edge pixel
        for i in 0..=radius {
            queue.push_back(first);

            sum_a += alpha(first) * (i + 1);
            sum_r += red(first) * (i + 1);
            sum_g += green(first) * (i + 1);
            sum_b += blue(first) * (i + 1);
        }
        let mut sum_out_a = alpha(first) * (radius + 1);
        let mut sum_out_r = red(first) * (radius + 1);
        let mut sum_out_g = green(first) * (radius + 1);
        let mut sum_out_b = blue(first) * (radius + 1);

        let (mut sum_in_a, mut sum_in_r, mut sum_in_g, mut sum_in_b) = (0, 0, 0, 0);
        // fill with starting pixels
        for (px, i) in row
            .iter()
            .copied()
            .skip(r + 1)
            .chain(iter::repeat(last))
            .take(r)
            .zip(1_u32..)
        {
            queue.push_back(px);

            sum_a += alpha(px) * (radius + 1 - i);
            sum_r += red(px) * (radius + 1 - i);
            sum_g += green(px) * (radius + 1 - i);
            sum_b += blue(px) * (radius + 1 - i);

            sum_in_a += alpha(px);
            sum_in_r += red(px);
            sum_in_g += green(px);
            sum_in_b += blue(px);
        }

        debug_assert_eq!(queue.len(), 2 * r + 1);

        for (i, r) in (0..width).zip(r..) {
            // set pixel
            let px = row.get_mut(i).unwrap();
            *px = pixel(sum_a / div, sum_r / div, sum_g / div, sum_b / div);

            let left = queue.pop_front().unwrap();
            let center = queue[queue.len() / 2];
            let right = *row.get(r).unwrap_or(&last);

            sum_a -= sum_out_a;
            sum_r -= sum_out_r;
            sum_g -= sum_out_g;
            sum_b -= sum_out_b;

            sum_out_a -= alpha(left);
            sum_out_r -= red(left);
            sum_out_g -= green(left);
            sum_out_b -= blue(left);

            sum_in_a += alpha(right);
            sum_in_r += red(right);
            sum_in_g += green(right);
            sum_in_b += blue(right);

            sum_a += sum_in_a;
            sum_r += sum_in_r;
            sum_g += sum_in_g;
            sum_b += sum_in_b;

            sum_out_a += alpha(center);
            sum_out_r += red(center);
            sum_out_g += green(center);
            sum_out_b += blue(center);

            sum_in_a -= alpha(center);
            sum_in_r -= red(center);
            sum_in_g -= green(center);
            sum_in_b -= blue(center);

            queue.push_back(right);
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
    fn empty_slice() {
        let mut empty: &mut [u32] = &mut [];

        let w = NonZeroUsize::new(1).unwrap();
        let h = NonZeroUsize::new(1).unwrap();
        let r = NonZeroU8::new(1).unwrap();

        blur(&mut empty, w, h, r);

        dbg!(empty);
    }

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

    #[test]
    fn incorrect_dimensions() {
        let mut v = vec![0x12345678; 9];
        let w = NonZeroUsize::new(5).unwrap();
        let h = NonZeroUsize::new(5).unwrap();
        let r = NonZeroU8::new(15).unwrap();

        blur(&mut v, w, h, r);

        dbg!(v);
    }
}
