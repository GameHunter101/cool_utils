#![allow(clippy::reversed_empty_ranges)]

use std::{
    borrow::BorrowMut,
    fmt::Debug,
    ops::{Index, IndexMut, Range},
};

pub struct RingBuffer<T> {
    pub buffer: Box<[T]>,
    ring_start: i32,
}

#[allow(unused)]
impl<T: Debug> RingBuffer<T> {
    pub fn new(data: Vec<T>) -> Self {
        RingBuffer {
            buffer: data.into_boxed_slice(),
            ring_start: 0,
        }
    }
    pub fn rotate_left(&mut self, amount: u32) {
        self.ring_start += amount as i32;
    }

    pub fn rotate_right(&mut self, amount: u32) {
        self.ring_start -= amount as i32;
    }

    pub fn replace_first(&mut self, element: T) {
        self[0] = element;
    }

    pub fn replace_last(&mut self, element: T) {
        self[-1] = element;
    }

    fn recalculate_index(&self, index: i32) -> usize {
        let mut index = ((index + self.ring_start) % self.buffer.len() as i32);
        if index < 0 {
            index += self.buffer.len() as i32;
        }
        index as usize
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn index_range(&self, range: Range<i32>) -> Vec<&T> {
        let start = self.recalculate_index(range.start);
        let end = self.recalculate_index(range.end);
        let mut returned_data: Vec<&T> = vec![]; //Vec::with_capacity(capacity);

        if start > end {
            let (end_chain, start_chain) = self.buffer.split_at(start);
            returned_data = start_chain.iter().chain(end_chain).collect();
            return returned_data;
        }
        for element in &self.buffer[start..=end] {
            returned_data.push(element);
        }
        returned_data
    }

    pub fn index_range_mut(&mut self, range: Range<i32>) -> Vec<&mut T> {
        let start = self.recalculate_index(range.start);
        let end = self.recalculate_index(range.end);
        let mut returned_data: Vec<&mut T> = vec![];

        if start > end {
            let (end_chain, start_chain) = self.buffer.split_at_mut(start);
            returned_data = start_chain.iter_mut().chain(end_chain).collect();
            return returned_data;
        }

        for element in self.buffer[start..=end].iter_mut() {
            returned_data.push(element);
        }
        returned_data
    }
}

impl<T: Debug> Debug for RingBuffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ordered = &self.index_range(0..-1);
        write!(f, "{:?}", ordered)
    }
}

impl<T: Debug> Index<i32> for RingBuffer<T> {
    fn index(&self, index: i32) -> &Self::Output {
        &self.buffer[self.recalculate_index(index)]
    }
    type Output = T;
}

impl<T: Debug> Index<u32> for RingBuffer<T> {
    fn index(&self, index: u32) -> &Self::Output {
        &self.buffer[index as usize]
    }
    type Output = T;
}

impl<T: Debug> IndexMut<i32> for RingBuffer<T> {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        &mut self.buffer[self.recalculate_index(index)]
    }
}

impl<T> FromIterator<T> for RingBuffer<T> {
    fn from_iter<B: IntoIterator<Item = T>>(iter: B) -> Self {
        RingBuffer {
            buffer: iter.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            ring_start: 0,
        }
    }
}

impl<T: Clone> From<&Vec<T>> for RingBuffer<T> {
    fn from(value: &Vec<T>) -> Self {
        RingBuffer {
            buffer: value.clone().into_boxed_slice(),
            ring_start: 0,
        }
    }
}

impl<'a, T: Debug> IntoIterator for &'a RingBuffer<T> {
    type Item = &'a T;
    type IntoIter = std::vec::IntoIter<&'a T>;
    fn into_iter(self) -> Self::IntoIter {
        let slice = self.index_range(0..-1);
        slice.into_iter()
    }
}

impl<'a, T: Debug> IntoIterator for &'a mut RingBuffer<T> {
    type Item = &'a mut T;
    type IntoIter = std::vec::IntoIter<&'a mut T>;
    fn into_iter(self) -> Self::IntoIter {
        self.index_range_mut(0..-1).into_iter()
    }
}

pub struct RingBuffer2D<T> {
    pub buffer: RingBuffer<RingBuffer<T>>,
    pub horizontal_start: i32,
    pub vertical_start: i32,
    pub rows: usize,
    pub cols: usize,
}

#[allow(unused)]
impl<T: Clone + Debug> RingBuffer2D<T> {
    pub fn new(data: Vec<Vec<T>>) -> Self {
        let rows = data[0].len();
        let cols = data.len();
        RingBuffer2D {
            buffer: data.into_iter().map(|i| RingBuffer::new(i)).collect(),
            horizontal_start: 0,
            vertical_start: 0,
            rows,
            cols,
        }
    }

    pub fn create_blank(rows: usize, cols: usize) -> Self {
        RingBuffer2D {
            buffer: (0..cols)
                .map(|_| RingBuffer::new(Vec::<T>::with_capacity(rows)))
                .collect(),
            horizontal_start: 0,
            vertical_start: 0,
            rows,
            cols,
        }
    }

    pub fn rotate_left(&mut self, amount: u32) {
        self.horizontal_start += amount as i32;
        self.horizontal_start %= self.cols as i32;
        self.buffer.rotate_left(amount);
    }

    pub fn rotate_right(&mut self, amount: u32) {
        self.horizontal_start -= amount as i32;
        self.horizontal_start %= self.cols as i32;
        self.buffer.rotate_right(amount);
    }

    pub fn rotate_up(&mut self, amount: u32) {
        self.vertical_start -= amount as i32;
        self.vertical_start %= self.rows as i32;
        for vertical_slice in &mut self.buffer {
            vertical_slice.rotate_right(amount);
        }
    }

    pub fn rotate_down(&mut self, amount: u32) {
        self.vertical_start += amount as i32;
        self.vertical_start %= self.rows as i32;
        for vertical_slice in &mut self.buffer {
            vertical_slice.rotate_left(amount);
        }
    }

    /// Replace the first vertical slice
    pub fn replace_first(&mut self, element: RingBuffer<T>) {
        self[0] = element;
    }

    // Replace the last vertical slice
    pub fn replace_last(&mut self, element: RingBuffer<T>) {
        self[-1] = element;
    }

    pub fn recalculate_index_horizontal(&self, index: i32) -> usize {
        let mut index = ((index + self.horizontal_start) % self.cols as i32);
        if index < 0 {
            index = self.cols as i32 - index.abs() ;
        }
        index as usize
    }

    pub fn recalculate_index_vertical(&self, index: i32) -> usize {
        let mut index = ((index + self.vertical_start) % self.rows as i32);
        if index < 0 {
            index = self.rows as i32 - index.abs() ;
        }
        index as usize
    }

    pub fn index_horizontal(&self, index: i32) -> Vec<T> {
        let mut slice: Vec<T> = Vec::with_capacity(self.buffer.len());
        for vertical_slice in self.into_iter() {
            slice.push(vertical_slice[index].clone());
        }
        slice
    }

    pub fn mut_index_horizontal(&mut self, index: i32, data: &[T]) {
        for (i, vertical_slice) in self.buffer.borrow_mut().into_iter().enumerate() {
            vertical_slice[index] = data[i].clone();
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn flatten(&self) -> Vec<T> {
        let mut size = self.into_iter().map(|e| e.len()).sum();
        let mut flattened = Vec::with_capacity(size);
        for z_slice in self.into_iter() {
            for e in z_slice.into_iter() {
                flattened.push(e.clone());
            }
        }
        flattened
    }

    pub fn linearize_index(&self, index_2d: (i32, i32)) -> usize {
        // println!("HORIZONTAL_START: {}, {}", self.recalculate_index(index_2d.0), index_2d.0);
        self.rows * self.recalculate_index_horizontal(index_2d.0)
            + self.recalculate_index_vertical(index_2d.1)
    }
}

impl<T: Clone + Copy + Debug> RingBuffer2D<T> {
    pub fn create_full(item: T, rows: usize, cols: usize) -> Self {
        RingBuffer2D {
            buffer: (0..cols)
                .map(|_| RingBuffer::new(vec![item; rows]))
                .collect(),
            horizontal_start: 0,
            vertical_start: 0,
            rows,
            cols,
        }
    }
}

/// Index gives vertical slices
impl<T: Debug> Index<i32> for RingBuffer2D<T> {
    fn index(&self, index: i32) -> &Self::Output {
        &self.buffer[index]
    }
    type Output = RingBuffer<T>;
}

impl<T: Debug> Index<u32> for RingBuffer2D<T> {
    fn index(&self, index: u32) -> &Self::Output {
        &self.buffer[index]
    }
    type Output = RingBuffer<T>;
}

/// Mutate vertical slices
impl<T: Debug> IndexMut<i32> for RingBuffer2D<T> {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        &mut self.buffer[index]
    }
}

impl<T: Debug> Index<(i32, i32)> for RingBuffer2D<T> {
    fn index(&self, index: (i32, i32)) -> &Self::Output {
        &self.buffer[index.0][index.1]
    }
    type Output = T;
}

impl<T: Debug> IndexMut<(i32, i32)> for RingBuffer2D<T> {
    fn index_mut(&mut self, index: (i32, i32)) -> &mut Self::Output {
        &mut self.buffer[index.0][index.1]
    }
}

impl<'a, T: Debug> IntoIterator for &'a RingBuffer2D<T> {
    type Item = &'a RingBuffer<T>;
    type IntoIter = std::vec::IntoIter<&'a RingBuffer<T>>;
    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter()
    }
}
