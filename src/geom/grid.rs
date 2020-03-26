//! Generic grids

use std::clone::Clone;
use std::slice::{Iter, IterMut};

/// A generic 2d grid.
///
/// Flat representation in memory.
pub struct Grid<T: Clone> {
    width: usize,
    height: usize,
    data: Vec<T>,
}

impl<T: Clone> Grid<T> {
    #[allow(dead_code)]
    pub fn new(width: usize, height: usize, default: T) -> Grid<T> {
        let mut grid = Grid {
            width,
            height,
            data: Vec::new(),
        };
        grid.data.resize(width * height, default);
        grid
    }

    /// Width (x dimension) of the grid in cells.
    #[allow(dead_code)]
    pub fn width(&self) -> i32 {
        self.width as i32
    }

    /// Height (y dimension) of the grid in cells.
    #[allow(dead_code)]
    pub fn height(&self) -> i32 {
        self.height as i32
    }

    /// Returns an immutable reference to the cell at x, y.
    #[allow(dead_code)]
    pub fn get(&self, x: usize, y: usize) -> &T {
        &self.data[self.index(x, y)]
    }

    /// Returns a mutable reference to the cell at x, y.
    #[allow(dead_code)]
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        let idx = self.index(x, y);
        &mut self.data[idx]
    }

    /// Returns an immutable reference to the cell at index (i.e. index in flat memory).
    #[allow(dead_code)]
    pub fn get_index(&self, i: usize) -> &T {
        &self.data[i]
    }

    /// Returns a mutable reference to the cell at index (i.e. index in flat memory).
    #[allow(dead_code)]
    pub fn get_index_mut(&mut self, i: usize) -> &mut T {
        &mut self.data[i]
    }

    /// Sets the cell at x, y.
    #[allow(dead_code)]
    pub fn set(&mut self, x: usize, y: usize, value: T) {
        let idx = self.index(x, y);
        self.data[idx] = value;
    }

    /// Sets the cell at index (i.e. index in flat memory).
    #[allow(dead_code)]
    pub fn set_index(&mut self, i: usize, value: T) {
        self.data[i] = value;
    }

    /// Returns if the grid contains coordinate (x, y).
    #[allow(dead_code)]
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width as i32 && y < self.width as i32
    }

    /// Calculates memory index from x, y coordinates.
    pub fn index(&self, x: usize, y: usize) -> usize {
        x * self.height + y
    }

    /// Calculates x, y coordinates from memory index.
    pub fn coord(&self, index: usize) -> (i32, i32) {
        ((index / self.height) as i32, (index % self.height) as i32)
    }

    /// Fills the grid using a closure with coordinates as arguments.
    #[allow(dead_code)]
    pub fn fill_xy<F>(&mut self, f: F)
    where
        F: Fn(usize, usize) -> T,
    {
        for x in 0..self.width {
            for y in 0..self.height {
                let idx = self.index(x, y);
                self.data[idx] = f(x, y);
            }
        }
    }

    /// Fills the grid using a closure without arguments.
    #[allow(dead_code)]
    pub fn fill<F>(&mut self, f: F)
    where
        F: Fn() -> T,
    {
        for x in 0..self.width {
            for y in 0..self.height {
                let idx = self.index(x, y);
                self.data[idx] = f();
            }
        }
    }

    /// Returns an Iterator over all grid cells in memory order.
    pub fn iter(&self) -> Iter<T> {
        self.data.iter()
    }

    /// Returns a mutable Iterator over all grid cells in memory order.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.data.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn create_grid() {
        let grid = crate::geom::grid::Grid::new(10, 10, 0);
        let v = grid.get(0, 0);
        assert_eq!(*v, 0);
    }

    #[test]
    fn set_value() {
        let mut grid = crate::geom::grid::Grid::new(10, 10, 0);
        grid.set(1, 2, 3);
        let v = grid.get(1, 2);
        assert_eq!(*v, 3);
    }

    #[test]
    fn fill() {
        let mut grid = crate::geom::grid::Grid::new(10, 10, 0);
        grid.fill_xy(|x, y| x + y);
        assert_eq!(*grid.get(3, 2), 3 + 2);
        assert_eq!(*grid.get(8, 3), 8 + 3);
    }

    #[test]
    fn contains() {
        let grid = crate::geom::grid::Grid::new(10, 10, 0);
        assert!(grid.contains(9, 9));
        assert!(!grid.contains(10, 10));
    }
}
