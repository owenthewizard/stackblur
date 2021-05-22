// Thanks to The0x539#3295

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
}

impl<T> GridSlice<T> for [T] {
    fn column(&self, x: usize, width: usize) -> SliceColumn<'_, T> {
        self.iter().column(x, width)
    }
    
    fn column_mut(&mut self, x: usize, width: usize) -> SliceColumnMut<'_, T> {
        self.iter_mut().column(x, width)
    }
}
