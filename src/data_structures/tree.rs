use std::{
    fmt::{Debug, write},
    ops::{Index, IndexMut},
};

#[derive(Debug)]
pub enum TreeError {
    IndexOutOfBoundsError,
}

#[derive(Debug)]
pub struct Tree<T> {
    node: T,
    children: Vec<Tree<T>>,
}

impl<T: Debug> Tree<T> {
    pub fn new(root: T) -> Self {
        Self {
            node: root,
            children: Vec::new(),
        }
    }

    pub fn node(&self) -> &T {
        &self.node
    }

    pub fn children(&self) -> &Vec<Tree<T>> {
        &self.children
    }

    pub fn add_child_node(&mut self, node: T) {
        self.add_child_tree(Tree::new(node));
    }

    pub fn add_child_tree(&mut self, tree: Tree<T>) {
        self.children.push(tree)
    }

    pub fn max_depth(&self) -> usize {
        if self.is_leaf() {
            return 1;
        }
        let mut children_length = self
            .children
            .iter()
            .map(|child| child.max_depth())
            .collect::<Vec<_>>();
        children_length.sort();
        return 1 + children_length.last().unwrap();
    }

    pub fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }

    pub fn index_depth(&self, index: Vec<usize>) -> Result<&Self, TreeError> {
        let mut current_node = self;
        for i in index {
            if current_node.children.len() == 0 {
                return Err(TreeError::IndexOutOfBoundsError);
            }
            current_node = &current_node[i];
        }
        Ok(current_node)
    }

    pub fn index_mut_depth(&mut self, index: Vec<usize>) -> Result<&mut Self, TreeError> {
        let mut current_node = self;
        for i in index {
            if current_node.children.len() == 0 {
                return Err(TreeError::IndexOutOfBoundsError);
            }
            current_node = &mut current_node[i];
        }
        Ok(current_node)
    }

    pub fn append_at_depth(&mut self, index: Vec<usize>, node: T) -> Result<(), TreeError> {
        self.index_mut_depth(index)?.children.push(Tree::new(node));
        Ok(())
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

#[test]
fn tree_test() {
    let mut tree = Tree::new(0);

    for i in 0..3 {
        tree.add_child_node(i);
        for j in 0..2 {
            tree[i].add_child_node(8);
        }
    }
    tree.append_at_depth(vec![0], 324);
    dbg!(&tree);
    let val = tree.index_depth(vec![0,2]).unwrap().node();
    assert_eq!(324, *val);
}
