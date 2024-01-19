use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Tree<T> {
    content: T,
    nodes: Vec<Tree<T>>
}

impl<T> Index<usize> for Tree<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index].content
    }
}

impl<T> IndexMut<usize> for Tree<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index].content
    }
}

impl<T> Index<usize> for Tree<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index].content
    }
}

impl<T> IndexMut<usize> for Tree<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index].content
    }
}
