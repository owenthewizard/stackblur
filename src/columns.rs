// Thanks to The0x539#3295

use std::mem::transmute;

type Column<T> = std::iter::StepBy<std::iter::Skip<T>>;
type SliceColumn<'a, T> = Column<std::slice::Iter<'a, T>>;
type SliceColumnMut<'a, T> = Column<std::slice::IterMut<'a, T>>;

pub trait ColumnIter: Iterator + Sized {
    fn column(self, x: usize, width: usize) -> Column<Self> {
        self.skip(x).step_by(width)
    }
}

impl<T: Iterator> ColumnIter for T {}

pub trait GridSlice<T> {
    fn column(&self, x: usize, width: usize) -> SliceColumn<'_, T>;
    fn column_mut(&mut self, x: usize, width: usize) -> SliceColumnMut<'_, T>;
    fn columns(&self, width: usize) -> Columns<'_, T>;
    unsafe fn columns_mut(&mut self, width: usize) -> ColumnsMut<'_, T>;
}

impl<T> GridSlice<T> for [T] {
    fn column(&self, x: usize, width: usize) -> SliceColumn<'_, T> {
        self.iter().column(x, width)
    }

    fn column_mut(&mut self, x: usize, width: usize) -> SliceColumnMut<'_, T> {
        self.iter_mut().column(x, width)
    }

    fn columns(&self, width: usize) -> Columns<'_, T> {
        Columns::new(self, width)
    }

    unsafe fn columns_mut(&mut self, width: usize) -> ColumnsMut<'_, T> {
        ColumnsMut::new(self, width)
    }
}

#[derive(Debug)]
pub struct Columns<'a, T: 'a> {
    v: &'a [T],
    width: usize,
    x: usize,
}

#[derive(Debug)]
pub struct ColumnsMut<'a, T: 'a> {
    v: &'a mut [T],
    width: usize,
    x: usize,
}

impl<'a, T: 'a> Columns<'a, T> {
    pub const fn new(slice: &'a [T], width: usize) -> Self {
        Self {
            v: slice,
            width,
            x: 0,
        }
    }
}

impl<'a, T: 'a> ColumnsMut<'a, T> {
    pub unsafe fn new(slice: &'a mut [T], width: usize) -> Self {
        Self {
            v: slice,
            width,
            x: 0,
        }
    }
}

impl<'a, T> Iterator for Columns<'a, T> {
    type Item = SliceColumn<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.v.is_empty() || self.x == self.width {
            None
        } else {
            self.x += 1;
            Some(self.v.column(self.x - 1, self.width))
        }
    }
}

impl<'long, T> Iterator for ColumnsMut<'long, T> {
    type Item = SliceColumnMut<'long, T>;

    fn next<'short>(&'short mut self) -> Option<Self::Item> {
        if self.v.is_empty() || self.x == self.width {
            None
        } else {
            self.x += 1;
            let ret = self.v.column_mut(self.x - 1, self.width);
            let ret = unsafe { transmute::<SliceColumnMut<'short, T>, Self::Item>(ret) };
            Some(ret)
        }
    }
}
