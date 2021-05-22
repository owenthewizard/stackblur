use std::collections::VecDeque;
use std::convert::From;

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
    pub fn sum_r(&self) -> u32 {
        let n = ((self.0.len() / 2) + 1) as u32;
        let mut iter = self.0.iter().copied();

        let a = iter
            .by_ref()
            .take(n as usize)
            .zip(1_u32..)
            .map(|(v, i)| red(v) * i)
            .sum::<u32>();

        let b = iter
            .take(n as usize - 1)
            .zip((1..n).rev())
            .map(|(v, i)| red(v) * i)
            .sum::<u32>();

        a + b
    }

    pub fn sum_g(&self) -> u32 {
        let n = ((self.0.len() / 2) + 1) as u32;
        let mut iter = self.0.iter().copied();

        let a = iter
            .by_ref()
            .take(n as usize)
            .zip(1_u32..)
            .map(|(v, i)| green(v) * i)
            .sum::<u32>();

        let b = iter
            .take(n as usize - 1)
            .zip((1..n).rev())
            .map(|(v, i)| green(v) * i)
            .sum::<u32>();

        a + b
    }
    pub fn sum_b(&self) -> u32 {
        let n = ((self.0.len() / 2) + 1) as u32;
        let mut iter = self.0.iter().copied();

        let a = iter
            .by_ref()
            .take(n as usize)
            .zip(1_u32..)
            .map(|(v, i)| blue(v) * i)
            .sum::<u32>();

        let b = iter
            .take(n as usize - 1)
            .zip((1..n).rev())
            .map(|(v, i)| blue(v) * i)
            .sum::<u32>();

        a + b
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
