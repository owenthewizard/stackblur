use std::cmp::min;
use std::collections::VecDeque;
use std::convert::From;

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

#[derive(Default, Debug)]
pub struct BlurStack(VecDeque<u32>);

impl BlurStack {
    pub fn sums(&self) -> (u32, u32, u32, u32) {
        let (mut a, mut r, mut g, mut b) = (0, 0, 0, 0);
        let n = self.0.len() as u32;
        for (&v, i) in self.0.iter().zip(0_u32..) {
            let i = min(i + 1, n - i);
            a += alpha(v) * i;
            r += red(v) * i;
            g += green(v) * i;
            b += blue(v) * i;
        }
        (a, r, g, b)
    }

    pub fn pop_front(&mut self) -> Option<u32> {
        self.0.pop_front()
    }

    pub fn push_front(&mut self, e: u32) {
        self.0.push_front(e)
    }

    pub fn push_back(&mut self, e: u32) {
        self.0.push_back(e)
    }

    pub fn with_capacity(n: usize) -> Self {
        Self(VecDeque::with_capacity(n as usize))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<VecDeque<u32>> for BlurStack {
    fn from(v: VecDeque<u32>) -> Self {
        Self(v)
    }
}
