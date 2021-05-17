use std::cmp::min;
use std::num::{NonZeroU8, NonZeroUsize};

#[cfg(test)]
use mutagen::mutate;

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
#[cfg_attr(test, mutate)]
pub fn blur(src: &mut [u32], width: NonZeroUsize, height: NonZeroUsize, radius: NonZeroU8) {
    blur_horiz(src, width, height, radius);
    blur_vert(src, width, height, radius);
}

/// Performs a horizontal pass of stackblur.
/// Input is expected to be in linear RGB color space.
#[cfg_attr(test, mutate)]
pub fn blur_horiz(src: &mut [u32], width: NonZeroUsize, height: NonZeroUsize, radius: NonZeroU8) {
    let width = width.get();
    let height = height.get();
    let radius = u32::from(min(radius.get(), 254));
    let r = radius as usize;

    let wm = width - 1;
    let div = 2 * r + 1;
    let mul_sum = MUL_TABLE[r];
    let shr_sum = SHR_TABLE[r];
    let mut stack = vec![0; div];

    for y in 0..height {
        let mut sum_r = 0;
        let mut sum_g = 0;
        let mut sum_b = 0;

        let mut sum_out_r = 0;
        let mut sum_out_g = 0;
        let mut sum_out_b = 0;

        let mut src_i = width * y;
        let mut stack_i;

        for i in 0..=radius {
            stack_i = i as usize;
            stack[stack_i] = src[src_i];

            sum_r += red(src[src_i]) * (i + 1);
            sum_g += green(src[src_i]) * (i + 1);
            sum_b += blue(src[src_i]) * (i + 1);

            sum_out_r += red(src[src_i]);
            sum_out_g += green(src[src_i]);
            sum_out_b += blue(src[src_i]);
        }

        let mut sum_in_r = 0;
        let mut sum_in_g = 0;
        let mut sum_in_b = 0;

        for i in 1..=radius {
            if i as usize <= wm {
                src_i += 1
            }

            stack_i = i as usize + r;
            stack[stack_i] = src[src_i];

            sum_r += red(src[src_i]) * (radius + 1 - i);
            sum_g += green(src[src_i]) * (radius + 1 - i);
            sum_b += blue(src[src_i]) * (radius + 1 - i);

            sum_in_r += red(src[src_i]);
            sum_in_g += green(src[src_i]);
            sum_in_b += blue(src[src_i]);
        }

        let mut sp = r;
        let mut xp = min(r, wm);

        src_i = xp + y * width;
        let mut dst_i = y * width;

        for _x in 0..width {
            src[dst_i] = pixel(
                (sum_r * mul_sum) >> shr_sum,
                (sum_g * mul_sum) >> shr_sum,
                (sum_b * mul_sum) >> shr_sum,
            );
            dst_i += 1;

            sum_r -= sum_out_r;
            sum_g -= sum_out_g;
            sum_b -= sum_out_b;

            let mut stack_start = sp + div - r;
            if stack_start >= div {
                stack_start -= div
            }
            stack_i = stack_start;

            sum_out_r -= red(stack[stack_i]);
            sum_out_g -= green(stack[stack_i]);
            sum_out_b -= blue(stack[stack_i]);

            if xp < wm {
                src_i += 1;
                xp += 1;
            }

            stack[stack_i] = src[src_i];

            sum_in_r += red(src[src_i]);
            sum_in_g += green(src[src_i]);
            sum_in_b += blue(src[src_i]);

            sum_r += sum_in_r;
            sum_g += sum_in_g;
            sum_b += sum_in_b;

            sp += 1;
            if sp >= div {
                sp = 0
            }
            stack_i = sp;

            sum_out_r += red(stack[stack_i]);
            sum_out_g += green(stack[stack_i]);
            sum_out_b += blue(stack[stack_i]);

            sum_in_r -= red(stack[stack_i]);
            sum_in_g -= green(stack[stack_i]);
            sum_in_b -= blue(stack[stack_i]);
        }
    }
}

/// Performs a vertical pass of stackblur.
/// Input is expected to be in linear RGB color space.
#[cfg_attr(test, mutate)]
pub fn blur_vert(src: &mut [u32], width: NonZeroUsize, height: NonZeroUsize, radius: NonZeroU8) {
    let width = width.get();
    let height = height.get();
    let radius = u32::from(min(radius.get(), 254));
    let r = radius as usize;

    let hm = height - 1;
    let div = 2 * r + 1;
    let mul_sum = MUL_TABLE[r];
    let shr_sum = SHR_TABLE[r];
    let mut stack = vec![0; div];

    for x in 0..width {
        let mut sum_r = 0;
        let mut sum_g = 0;
        let mut sum_b = 0;

        let mut sum_out_r = 0;
        let mut sum_out_g = 0;
        let mut sum_out_b = 0;

        let mut src_i = x;
        let mut stack_i;

        for i in 0..=radius {
            stack_i = i as usize;
            stack[stack_i] = src[src_i];

            sum_r += red(src[src_i]) * (i + 1);
            sum_g += green(src[src_i]) * (i + 1);
            sum_b += blue(src[src_i]) * (i + 1);

            sum_out_r += red(src[src_i]);
            sum_out_g += green(src[src_i]);
            sum_out_b += blue(src[src_i]);
        }

        let mut sum_in_r = 0;
        let mut sum_in_g = 0;
        let mut sum_in_b = 0;

        for i in 1..=radius {
            if i as usize <= hm {
                src_i += width
            }

            stack_i = i as usize + r;
            stack[stack_i] = src[src_i];

            sum_r += red(src[src_i]) * (radius + 1 - i);
            sum_g += green(src[src_i]) * (radius + 1 - i);
            sum_b += blue(src[src_i]) * (radius + 1 - i);

            sum_in_r += red(src[src_i]);
            sum_in_g += green(src[src_i]);
            sum_in_b += blue(src[src_i]);
        }

        let mut sp = r;
        let mut yp = min(r, hm);

        src_i = x + yp * width;
        let mut dst_i = x;

        for _y in 0..height {
            src[dst_i] = pixel(
                (sum_r * mul_sum) >> shr_sum,
                (sum_g * mul_sum) >> shr_sum,
                (sum_b * mul_sum) >> shr_sum,
            );
            dst_i += width;

            sum_r -= sum_out_r;
            sum_g -= sum_out_g;
            sum_b -= sum_out_b;

            let mut stack_start = sp + div - r;
            if stack_start >= div {
                stack_start -= div
            }
            stack_i = stack_start;

            sum_out_r -= red(stack[stack_i]);
            sum_out_g -= green(stack[stack_i]);
            sum_out_b -= blue(stack[stack_i]);

            if yp < hm {
                src_i += width;
                yp += 1;
            }

            stack[stack_i] = src[src_i];

            sum_in_r += red(src[src_i]);
            sum_in_g += green(src[src_i]);
            sum_in_b += blue(src[src_i]);

            sum_r += sum_in_r;
            sum_g += sum_in_g;
            sum_b += sum_in_b;

            sp += 1;
            if sp >= div {
                sp = 0;
            }
            stack_i = sp;

            sum_out_r += red(stack[stack_i]);
            sum_out_g += green(stack[stack_i]);
            sum_out_b += blue(stack[stack_i]);

            sum_in_r -= red(stack[stack_i]);
            sum_in_g -= green(stack[stack_i]);
            sum_in_b -= blue(stack[stack_i]);
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
        let w = NonZeroUsize::new(1);
        let h = NonZeroUsize::new(1);
        let r = NonZeroU8::new(1);

        blur(&mut v, w, h, r);

        dbg!(v);
    }

    #[test]
    fn tiny_image_large_radius() {
        let mut v = vec![0x12345678; 9];
        let w = NonZeroUsize::new(3);
        let h = NonZeroUsize::new(3);
        let r = NonZeroU8::new(u8::MAX);

        blur(&mut v, w, h, r);

        dbg!(v);
    }
}
