use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Tree<T> {
    node: T,
    children: Vec<Tree<T>>,
}

impl<T> Tree<T> {
    pub fn new(root: T) -> Self {
        Self {
            node: root,
            children: Vec::new(),
        }
    }

    pub fn node(&self) -> &T {
        &self.node
    }

    pub fn add_child_node(&mut self, node: T) {
        self.add_child_tree(Tree::new(node));
    }

    pub fn add_child_tree(&mut self, tree: Tree<T>) {
        self.children.push(tree)
    }

    pub fn max_depth(&self) -> usize {
        todo!("Do some depth-first-search to find the total depth of the current root tree")
    }
}

impl<T> Index<usize> for Tree<T> {
    type Output = Tree<T>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.children[index]
    }
}

impl<T> IndexMut<usize> for Tree<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.children[index]
    }
}

impl<T> Index<&[usize]> for Tree<T> {
    type Output = Tree<T>;
    fn index(&self, index: &[usize]) -> &Self::Output {
        let mut current_node = self;
        for i in index {
            current_node = &current_node[*i];
        }
        current_node
    }
}

impl<T> IndexMut<&[usize]> for Tree<T> {
    fn index_mut(&mut self, index: &[usize]) -> &mut Self::Output {
        let mut current_node = self;
        for i in index {
            current_node = &mut current_node[*i];
        }
        current_node
    }
}

#[test]
fn tree_test() {
    let mut tree = Tree::new(0);

    for i in 0..3 {
        tree.add_child_node(i);
        for j in 0..2 {
            tree[i].add_child_node(j);
        }
    }
    dbg!(tree);
}
