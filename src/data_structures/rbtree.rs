use std::{boxed, ptr::NonNull};

pub struct RBTree<T: Ord + std::fmt::Debug + Clone> {
    root: Link<T>,
    nil: NonNull<NilNode<T>>,
}

#[allow(unsafe_op_in_unsafe_fn)]
impl<T: Ord + std::fmt::Debug + Clone> RBTree<T> {
    pub fn new() -> Self {
        let nil = NilNode::new();
        Self {
            root: Link::Nil(nil),
            nil,
        }
    }

    pub unsafe fn unsafe_search(&mut self, element: &T) -> Option<NonNull<Node<T>>> {
        let mut traverse_node = self.root;
        while let Link::Real(node) = traverse_node
            && &(*node.as_ptr()).value != element
        {
            if element > &(*node.as_ptr()).value {
                traverse_node = (*node.as_ptr()).right;
            } else {
                traverse_node = (*node.as_ptr()).left;
            }
        }

        if let Link::Real(node) = traverse_node {
            Some(node)
        } else {
            None
        }
    }

    pub fn search(&mut self, element: &T) -> bool {
        unsafe { self.unsafe_search(&element).is_some() }
    }

    pub unsafe fn unsafe_insert(&mut self, element: T) -> NonNull<Node<T>> {
        let new_node = Node::new(element, self.nil(), self.nil());
        let mut traverse_target = self.root;
        let mut traverse_parent = self.nil();

        while let Link::Real(target) = traverse_target {
            traverse_parent = traverse_target;
            if (*new_node.as_ptr()).value < (*target.as_ptr()).value {
                traverse_target = (*target.as_ptr()).left;
            } else {
                traverse_target = (*target.as_ptr()).right;
            }
        }

        (*new_node.as_ptr()).parent = traverse_parent;

        if let Link::Nil(_) = traverse_parent {
            self.root = Link::Real(new_node)
        } else if let Link::Real(parent) = traverse_parent {
            if (*new_node.as_ptr()).value < (*parent.as_ptr()).value {
                (*parent.as_ptr()).left = Link::Real(new_node);
            } else {
                (*parent.as_ptr()).right = Link::Real(new_node);
            }
        }

        let mut rule_violator = new_node;

        while Link::Real(rule_violator) != self.root
            && (*rule_violator.as_ptr()).parent.color() == Color::Red
        {
            let mut parent = (*rule_violator.as_ptr()).parent.into_node();
            let mut grandparent = (*parent.as_ptr()).parent.into_node();
            if Link::Real(parent) == (*grandparent.as_ptr()).left {
                let uncle = (*grandparent.as_ptr()).right;
                if uncle.color() == Color::Red {
                    (*parent.as_ptr()).color = Color::Black;
                    (*uncle.into_node().as_ptr()).color = Color::Black;
                    (*grandparent.as_ptr()).color = Color::Red;
                    rule_violator = grandparent;
                } else {
                    if Link::Real(rule_violator) == (*parent.as_ptr()).right {
                        rule_violator = parent;
                        self.rotate_left(rule_violator);
                    }

                    parent = (*rule_violator.as_ptr()).parent.into_node();
                    grandparent = (*parent.as_ptr()).parent.into_node();

                    (*parent.as_ptr()).color = Color::Black;
                    (*grandparent.as_ptr()).color = Color::Red;
                    self.rotate_right(grandparent);
                }
            } else {
                let uncle = (*grandparent.as_ptr()).left;
                if uncle.color() == Color::Red {
                    (*parent.as_ptr()).color = Color::Black;
                    (*uncle.into_node().as_ptr()).color = Color::Black;
                    (*grandparent.as_ptr()).color = Color::Red;
                    rule_violator = grandparent;
                } else {
                    if Link::Real(rule_violator) == (*parent.as_ptr()).left {
                        rule_violator = parent;
                        self.rotate_right(rule_violator);
                    }

                    parent = (*rule_violator.as_ptr()).parent.into_node();
                    grandparent = (*parent.as_ptr()).parent.into_node();

                    (*parent.as_ptr()).color = Color::Black;
                    (*grandparent.as_ptr()).color = Color::Red;
                    self.rotate_left(grandparent);
                }
            }
        }

        if let Link::Real(root) = self.root {
            (*root.as_ptr()).color = Color::Black;
        }

        new_node
    }

    pub fn insert(&mut self, element: T) {
        unsafe {
            self.unsafe_insert(element);
        }
    }

    pub fn delete(&mut self, element: &T) -> bool {
        unsafe {
            let mut traversed_node = self.root;
            while let Link::Real(node) = traversed_node
                && &(*node.as_ptr()).value != element
            {
                if element > &(*node.as_ptr()).value {
                    traversed_node = (*node.as_ptr()).right;
                } else {
                    traversed_node = (*node.as_ptr()).left;
                }
            }

            if traversed_node.is_nil() {
                return false;
            }

            let deletion_target = traversed_node.into_node();
            let spliced_node = if (*deletion_target.as_ptr()).left.is_nil()
                || (*deletion_target.as_ptr()).right.is_nil()
            {
                deletion_target
            } else {
                Node::in_order_successor(deletion_target)
            };

            let child_of_spliced_node =
                if let Link::Real(left_child) = (*spliced_node.as_ptr()).left {
                    Link::Real(left_child)
                } else {
                    (*spliced_node.as_ptr()).right
                };

            match child_of_spliced_node {
                Link::Real(real) => (*real.as_ptr()).parent = (*spliced_node.as_ptr()).parent,
                Link::Nil(nil) => {
                    (*nil.as_ptr()).parent =
                        if let Link::Real(parent) = (*spliced_node.as_ptr()).parent {
                            Some(parent)
                        } else {
                            None
                        }
                }
            }

            if let Link::Real(parent) = (*spliced_node.as_ptr()).parent {
                if (*parent.as_ptr()).left == Link::Real(spliced_node) {
                    (*parent.as_ptr()).left = child_of_spliced_node;
                } else {
                    (*parent.as_ptr()).right = child_of_spliced_node;
                }
            } else {
                self.root = child_of_spliced_node;
            }

            if spliced_node != deletion_target {
                (*deletion_target.as_ptr()).value = (*spliced_node.as_ptr()).value.clone();
            }

            if (*spliced_node.as_ptr()).color == Color::Black {
                self.delete_fix(child_of_spliced_node);
            }

            let _ = Box::from_raw(spliced_node.as_ptr());

            true
        }
    }

    fn delete_fix(&mut self, node: Link<T>) {
        unsafe {
            let mut double_black = node;
            while double_black != self.root && double_black.color() == Color::Black {
                let parent = double_black.parent().into_node();
                if double_black == (*parent.as_ptr()).left {
                    let mut sibling_of_double_black = (*parent.as_ptr()).right.into_node();
                    if (*sibling_of_double_black.as_ptr()).color == Color::Red {
                        (*sibling_of_double_black.as_ptr()).color = Color::Black;
                        (*parent.as_ptr()).color = Color::Red;
                        self.rotate_left(parent);

                        sibling_of_double_black = (*parent.as_ptr()).right.into_node();
                    }
                    if (*sibling_of_double_black.as_ptr()).left.color() == Color::Black
                        && (*sibling_of_double_black.as_ptr()).right.color() == Color::Black
                    {
                        (*sibling_of_double_black.as_ptr()).color = Color::Red;
                        double_black = double_black.parent();
                    } else {
                        if (*sibling_of_double_black.as_ptr()).right.color() == Color::Black {
                            if let Link::Real(left) = (*sibling_of_double_black.as_ptr()).left {
                                (*left.as_ptr()).color = Color::Black;
                            }
                            (*sibling_of_double_black.as_ptr()).color = Color::Red;
                            self.rotate_right(sibling_of_double_black);
                            sibling_of_double_black = (*parent.as_ptr()).right.into_node();
                        }

                        (*sibling_of_double_black.as_ptr()).color = (*parent.as_ptr()).color;
                        (*parent.as_ptr()).color = Color::Black;
                        (*(*sibling_of_double_black.as_ptr())
                            .right
                            .into_node()
                            .as_ptr())
                        .color = Color::Black;
                        self.rotate_left(parent);
                        double_black = self.root;
                    }
                } else {
                    let mut sibling_of_double_black = (*parent.as_ptr()).left.into_node();
                    if (*sibling_of_double_black.as_ptr()).color == Color::Red {
                        (*sibling_of_double_black.as_ptr()).color = Color::Black;
                        (*parent.as_ptr()).color = Color::Red;
                        self.rotate_right(parent);

                        sibling_of_double_black = (*parent.as_ptr()).left.into_node();
                    }
                    if (*sibling_of_double_black.as_ptr()).right.color() == Color::Black
                        && (*sibling_of_double_black.as_ptr()).left.color() == Color::Black
                    {
                        (*sibling_of_double_black.as_ptr()).color = Color::Red;
                        double_black = double_black.parent();
                    } else {
                        if (*sibling_of_double_black.as_ptr()).left.color() == Color::Black {
                            if let Link::Real(right) = (*sibling_of_double_black.as_ptr()).right {
                                (*right.as_ptr()).color = Color::Black;
                            }
                            (*sibling_of_double_black.as_ptr()).color = Color::Red;
                            self.rotate_left(sibling_of_double_black);
                            sibling_of_double_black = (*parent.as_ptr()).left.into_node();
                        }

                        (*sibling_of_double_black.as_ptr()).color = (*parent.as_ptr()).color;
                        (*parent.as_ptr()).color = Color::Black;
                        (*(*sibling_of_double_black.as_ptr())
                            .left
                            .into_node()
                            .as_ptr())
                        .color = Color::Black;
                        self.rotate_right(parent);
                        double_black = self.root;
                    }
                }
            }

            if let Link::Real(node) = double_black {
                (*node.as_ptr()).color = Color::Black;
            }
        }
    }

    fn rotate_left(&mut self, node: NonNull<Node<T>>) {
        unsafe {
            let new_left = node;
            let new_top = (*node.as_ptr()).right.into_node();
            (*new_left.as_ptr()).right = (*new_top.as_ptr()).left;
            if let Link::Real(right_child_of_new_left) = (*new_left.as_ptr()).right {
                (*right_child_of_new_left.as_ptr()).parent = Link::Real(new_left);
            }
            (*new_top.as_ptr()).parent = (*new_left.as_ptr()).parent;
            if (*new_left.as_ptr()).parent.is_nil() {
                self.root = Link::Real(new_top);
            } else if Link::Real(new_left)
                == (*(*new_left.as_ptr()).parent.into_node().as_ptr()).left
            {
                (*(*new_left.as_ptr()).parent.into_node().as_ptr()).left = Link::Real(new_top);
            } else {
                (*(*new_left.as_ptr()).parent.into_node().as_ptr()).right = Link::Real(new_top);
            }
            (*new_top.as_ptr()).left = Link::Real(new_left);
            (*new_left.as_ptr()).parent = Link::Real(new_top);
        }
    }

    fn rotate_right(&mut self, node: NonNull<Node<T>>) {
        unsafe {
            let new_right = node;
            let new_top = (*node.as_ptr()).left.into_node();
            (*new_right.as_ptr()).left = (*new_top.as_ptr()).right;
            if let Link::Real(left_child_of_new_right) = (*new_right.as_ptr()).left {
                (*left_child_of_new_right.as_ptr()).parent = Link::Real(new_right);
            }
            (*new_top.as_ptr()).parent = (*new_right.as_ptr()).parent;
            if (*new_right.as_ptr()).parent.is_nil() {
                self.root = Link::Real(new_top);
            } else if Link::Real(new_right)
                == (*(*new_right.as_ptr()).parent.into_node().as_ptr()).right
            {
                (*(*new_right.as_ptr()).parent.into_node().as_ptr()).right = Link::Real(new_top);
            } else {
                (*(*new_right.as_ptr()).parent.into_node().as_ptr()).left = Link::Real(new_top);
            }
            (*new_top.as_ptr()).right = Link::Real(new_right);
            (*new_right.as_ptr()).parent = Link::Real(new_top);
        }
    }

    fn nil(&self) -> Link<T> {
        Link::Nil(self.nil)
    }

    pub fn in_order_vec(&self) -> Vec<T> {
        if let Link::Real(root) = self.root {
            unsafe { root.as_ref().in_order_vec() }
        } else {
            Vec::new()
        }
    }

    pub fn print(&self) {
        unsafe {
            self.root.into_node().as_ref().print("", true, "");
        }
    }
}

impl<T: Ord + std::fmt::Debug + Clone> Drop for RBTree<T> {
    fn drop(&mut self) {
        if let Link::Real(root) = self.root {
            Node::node_drop(root);
        }

        unsafe {
            let _ = Box::from_raw(self.nil.as_ptr());
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Link<T: Ord + std::fmt::Debug> {
    Real(NonNull<Node<T>>),
    Nil(NonNull<NilNode<T>>),
}

impl<T: Ord + std::fmt::Debug> Copy for Link<T> {}

impl<T: Ord + std::fmt::Debug> Clone for Link<T> {
    fn clone(&self) -> Self {
        match self {
            Link::Real(ptr) => Link::Real(*ptr),
            Link::Nil(ptr) => Link::Nil(*ptr),
        }
    }
}

impl<T: Ord + std::fmt::Debug> Link<T> {
    fn color(&self) -> Color {
        unsafe {
            match self {
                Link::Real(real) => (*real.as_ptr()).color,
                Link::Nil(nil) => (*nil.as_ptr()).color,
            }
        }
    }

    fn into_node(self) -> NonNull<Node<T>> {
        unsafe {
            let Link::Real(node) = self else {
                panic!("Unwrapped node is nil")
            };

            node
        }
    }

    fn is_nil(&self) -> bool {
        match self {
            Link::Real(_) => false,
            Link::Nil(_) => true,
        }
    }

    fn parent(&self) -> Link<T> {
        unsafe {
            match self {
                Link::Real(real) => (*real.as_ptr()).parent,
                Link::Nil(nil) => match (*nil.as_ptr()).parent {
                    Some(parent) => Link::Real(parent),
                    None => Link::Nil(*nil),
                },
            }
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
enum Color {
    Red,
    Black,
}

#[derive(Debug)]
pub struct Node<T: Ord + std::fmt::Debug> {
    pub value: T,
    color: Color,
    left: Link<T>,
    right: Link<T>,
    parent: Link<T>,
}

pub struct NilNode<T: Ord + std::fmt::Debug> {
    color: Color,
    parent: Option<NonNull<Node<T>>>,
}

impl<T: Ord + std::fmt::Debug> NilNode<T> {
    fn new() -> NonNull<NilNode<T>> {
        unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(NilNode {
                color: Color::Black,
                parent: None,
            })))
        }
    }
}

impl<T: Ord + std::fmt::Debug + Clone> Node<T> {
    fn new(element: T, parent: Link<T>, nil: Link<T>) -> NonNull<Node<T>> {
        unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                value: element,
                color: Color::Red,
                left: nil,
                right: nil,
                parent,
            })))
        }
    }

    fn node_drop(node: NonNull<Node<T>>) {
        unsafe {
            let boxed_node = Box::from_raw(node.as_ptr());

            if let Link::Real(left) = boxed_node.left {
                Self::node_drop(left);
            }

            if let Link::Real(right) = boxed_node.right {
                Self::node_drop(right);
            }
        }
    }

    fn print(&self, indent: &str, is_final: bool, append: &str) {
        unsafe {
            println!("{indent}+- {:?} # {:?} - {append}", self.value, self.color);
            if let Link::Real(left) = self.left {
                let new_indent = format!("{indent}{}", if is_final { "   " } else { "|  " });
                left.as_ref().print(&new_indent, self.right.is_nil(), "L");
            }

            if let Link::Real(right) = self.right {
                let new_indent = format!("{indent}{}", if is_final { "   " } else { "|  " });
                right.as_ref().print(&new_indent, true, "R");
            }
        }
    }

    fn in_order_vec(&self) -> Vec<T> {
        unsafe {
            if self.left.is_nil() && self.right.is_nil() {
                return vec![self.value.clone()];
            }

            let left_vec = if let Link::Real(left) = self.left {
                left.as_ref().in_order_vec()
            } else {
                Vec::new()
            };
            let right_vec = if let Link::Real(right) = self.right {
                right.as_ref().in_order_vec()
            } else {
                Vec::new()
            };

            left_vec
                .into_iter()
                .chain(std::iter::once(self.value.clone()))
                .chain(right_vec.into_iter())
                .collect()
        }
    }

    fn in_order_successor(node: NonNull<Node<T>>) -> NonNull<Node<T>> {
        unsafe { Self::follow_left((*node.as_ptr()).right.into_node()) }
    }

    fn follow_left(node: NonNull<Node<T>>) -> NonNull<Node<T>> {
        unsafe {
            if let Link::Real(left) = (*node.as_ptr()).left {
                Node::follow_left(left)
            } else {
                node
            }
        }
    }

    fn height(&self) -> u32 {
        unsafe {
            if self.left.is_nil() && self.right.is_nil() {
                return 1;
            }

            let left_height = if let Link::Real(left) = self.left {
                left.as_ref().height()
            } else {
                0
            };

            let right_height = if let Link::Real(right) = self.right {
                right.as_ref().height()
            } else {
                0
            };

            left_height.max(right_height) + 1
        }
    }

    fn min_height(&self) -> u32 {
        unsafe {
            if self.left.is_nil() && self.right.is_nil() {
                return 1;
            }

            let left_height = if let Link::Real(left) = self.left {
                left.as_ref().min_height()
            } else {
                0
            };

            let right_height = if let Link::Real(right) = self.right {
                right.as_ref().min_height()
            } else {
                0
            };

            left_height.min(right_height) + 1
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use rand::distr::StandardUniform;

    use crate::data_structures::rbtree::Color;

    use super::{NilNode, Node, RBTree};

    #[test]
    fn successfully_construct_empty_tree() {
        let tree: RBTree<i32> = RBTree::new();
        assert!(tree.root.is_nil());
    }

    #[test]
    fn insert_case_1_left() {
        let mut tree = RBTree::new();

        tree.insert(0);
        tree.insert(2);
        tree.insert(-2);
        tree.insert(-1);

        assert_eq!(tree.in_order_vec(), vec![-2, -1, 0, 2]);

        let root = tree.root.into_node();
        unsafe {
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.into_node();
            assert_eq!((*left_child.as_ptr()).value, -2);
            assert_eq!((*left_child.as_ptr()).color, Color::Black);
            let right_child = (*root.as_ptr()).right.into_node();
            assert_eq!((*right_child.as_ptr()).value, 2);
            assert_eq!((*right_child.as_ptr()).color, Color::Black);
            let left_right_child = (*left_child.as_ptr()).right.into_node();
            assert_eq!((*left_right_child.as_ptr()).value, -1);
            assert_eq!((*left_right_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn insert_case_1_left_alternate() {
        let mut tree = RBTree::new();

        tree.insert(0);
        tree.insert(2);
        tree.insert(-2);
        tree.insert(-3);

        assert_eq!(tree.in_order_vec(), vec![-3, -2, 0, 2]);

        let root = tree.root.into_node();
        unsafe {
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.into_node();
            assert_eq!((*left_child.as_ptr()).value, -2);
            assert_eq!((*left_child.as_ptr()).color, Color::Black);
            let right_child = (*root.as_ptr()).right.into_node();
            assert_eq!((*right_child.as_ptr()).value, 2);
            assert_eq!((*right_child.as_ptr()).color, Color::Black);
            let left_left_child = (*left_child.as_ptr()).left.into_node();
            assert_eq!((*left_left_child.as_ptr()).value, -3);
            assert_eq!((*left_left_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn insert_case_1_right() {
        let mut tree = RBTree::new();

        tree.insert(0);
        tree.insert(2);
        tree.insert(-2);
        tree.insert(1);

        assert_eq!(tree.in_order_vec(), vec![-2, 0, 1, 2]);

        let root = tree.root.into_node();
        unsafe {
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.into_node();
            assert_eq!((*left_child.as_ptr()).value, -2);
            assert_eq!((*left_child.as_ptr()).color, Color::Black);
            let right_child = (*root.as_ptr()).right.into_node();
            assert_eq!((*right_child.as_ptr()).value, 2);
            assert_eq!((*right_child.as_ptr()).color, Color::Black);
            let right_left_child = (*right_child.as_ptr()).left.into_node();
            assert_eq!((*right_left_child.as_ptr()).value, 1);
            assert_eq!((*right_left_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn insert_case_1_right_alternate() {
        let mut tree = RBTree::new();

        tree.insert(0);
        tree.insert(2);
        tree.insert(-2);
        tree.insert(3);

        assert_eq!(tree.in_order_vec(), vec![-2, 0, 2, 3]);

        let root = tree.root.into_node();
        unsafe {
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.into_node();
            assert_eq!((*left_child.as_ptr()).value, -2);
            assert_eq!((*left_child.as_ptr()).color, Color::Black);
            let right_child = (*root.as_ptr()).right.into_node();
            assert_eq!((*right_child.as_ptr()).value, 2);
            assert_eq!((*right_child.as_ptr()).color, Color::Black);
            let right_right_child = (*right_child.as_ptr()).right.into_node();
            assert_eq!((*right_right_child.as_ptr()).value, 3);
            assert_eq!((*right_right_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn insert_case_2_then_3_left() {
        let mut tree = RBTree::new();
        tree.insert(0);
        tree.insert(2);
        tree.insert(-3);
        tree.insert(-1);
        tree.insert(-2);

        assert_eq!(tree.in_order_vec(), vec![-3, -2, -1, 0, 2]);

        let root = tree.root.into_node();
        unsafe {
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.into_node();
            assert_eq!((*left_child.as_ptr()).value, -2);
            assert_eq!((*left_child.as_ptr()).color, Color::Black);
            let right_child = (*root.as_ptr()).right.into_node();
            assert_eq!((*right_child.as_ptr()).value, 2);
            assert_eq!((*right_child.as_ptr()).color, Color::Black);
            let left_left_child = (*left_child.as_ptr()).left.into_node();
            assert_eq!((*left_left_child.as_ptr()).value, -3);
            assert_eq!((*left_left_child.as_ptr()).color, Color::Red);
            let left_right_child = (*left_child.as_ptr()).right.into_node();
            assert_eq!((*left_right_child.as_ptr()).value, -1);
            assert_eq!((*left_right_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn insert_case_2_then_3_left_at_root() {
        let mut tree = RBTree::new();
        tree.insert(0);
        tree.insert(-2);
        tree.insert(-1);

        assert_eq!(tree.in_order_vec(), vec![-2, -1, 0]);

        let root = tree.root.into_node();
        unsafe {
            assert_eq!((*root.as_ptr()).value, -1);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.into_node();
            assert_eq!((*left_child.as_ptr()).value, -2);
            assert_eq!((*left_child.as_ptr()).color, Color::Red);
            let right_child = (*root.as_ptr()).right.into_node();
            assert_eq!((*right_child.as_ptr()).value, 0);
            assert_eq!((*right_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn insert_case_2_then_3_right() {
        let mut tree = RBTree::new();
        tree.insert(0);
        tree.insert(3);
        tree.insert(-2);
        tree.insert(1);
        tree.insert(2);

        assert_eq!(tree.in_order_vec(), vec![-2, 0, 1, 2, 3]);

        let root = tree.root.into_node();
        unsafe {
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.into_node();
            assert_eq!((*left_child.as_ptr()).value, -2);
            assert_eq!((*left_child.as_ptr()).color, Color::Black);
            let right_child = (*root.as_ptr()).right.into_node();
            assert_eq!((*right_child.as_ptr()).value, 2);
            assert_eq!((*right_child.as_ptr()).color, Color::Black);
            let right_left_child = (*right_child.as_ptr()).left.into_node();
            assert_eq!((*right_left_child.as_ptr()).value, 1);
            assert_eq!((*right_left_child.as_ptr()).color, Color::Red);
            let right_right_child = (*right_child.as_ptr()).right.into_node();
            assert_eq!((*right_right_child.as_ptr()).value, 3);
            assert_eq!((*right_right_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn insert_case_2_then_3_right_at_root() {
        let mut tree = RBTree::new();
        tree.insert(0);
        tree.insert(2);
        tree.insert(1);

        assert_eq!(tree.in_order_vec(), vec![0, 1, 2]);

        let root = tree.root.into_node();
        unsafe {
            assert_eq!((*root.as_ptr()).value, 1);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.into_node();
            assert_eq!((*left_child.as_ptr()).value, 0);
            assert_eq!((*left_child.as_ptr()).color, Color::Red);
            let right_child = (*root.as_ptr()).right.into_node();
            assert_eq!((*right_child.as_ptr()).value, 2);
            assert_eq!((*right_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn insert_case_3_left() {
        let mut tree = RBTree::new();
        tree.insert(0);
        tree.insert(-2);
        tree.insert(3);
        tree.insert(2);
        tree.insert(1);

        assert_eq!(tree.in_order_vec(), vec![-2, 0, 1, 2, 3]);

        let root = tree.root.into_node();
        unsafe {
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.into_node();
            assert_eq!((*left_child.as_ptr()).value, -2);
            assert_eq!((*left_child.as_ptr()).color, Color::Black);
            let right_child = (*root.as_ptr()).right.into_node();
            assert_eq!((*right_child.as_ptr()).value, 2);
            assert_eq!((*right_child.as_ptr()).color, Color::Black);
            let right_left_child = (*right_child.as_ptr()).left.into_node();
            assert_eq!((*right_left_child.as_ptr()).value, 1);
            assert_eq!((*right_left_child.as_ptr()).color, Color::Red);
            let right_right_child = (*right_child.as_ptr()).right.into_node();
            assert_eq!((*right_right_child.as_ptr()).value, 3);
            assert_eq!((*right_right_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn insert_case_3_left_at_root() {
        let mut tree = RBTree::new();
        tree.insert(0);
        tree.insert(-1);
        tree.insert(-2);

        assert_eq!(tree.in_order_vec(), vec![-2, -1, 0]);

        let root = tree.root.into_node();
        unsafe {
            assert_eq!((*root.as_ptr()).value, -1);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.into_node();
            assert_eq!((*left_child.as_ptr()).value, -2);
            assert_eq!((*left_child.as_ptr()).color, Color::Red);
            let right_child = (*root.as_ptr()).right.into_node();
            assert_eq!((*right_child.as_ptr()).value, 0);
            assert_eq!((*right_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn insert_case_3_right() {
        let mut tree = RBTree::new();
        tree.insert(0);
        tree.insert(2);
        tree.insert(-3);
        tree.insert(-2);
        tree.insert(-1);

        assert_eq!(tree.in_order_vec(), vec![-3, -2, -1, 0, 2]);

        let root = tree.root.into_node();
        unsafe {
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.into_node();
            assert_eq!((*left_child.as_ptr()).value, -2);
            assert_eq!((*left_child.as_ptr()).color, Color::Black);
            let right_child = (*root.as_ptr()).right.into_node();
            assert_eq!((*right_child.as_ptr()).value, 2);
            assert_eq!((*right_child.as_ptr()).color, Color::Black);
            let left_left_child = (*left_child.as_ptr()).left.into_node();
            assert_eq!((*left_left_child.as_ptr()).value, -3);
            assert_eq!((*left_left_child.as_ptr()).color, Color::Red);
            let left_right_child = (*left_child.as_ptr()).right.into_node();
            assert_eq!((*left_right_child.as_ptr()).value, -1);
            assert_eq!((*left_right_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn insert_case_3_right_at_root() {
        let mut tree = RBTree::new();
        tree.insert(0);
        tree.insert(1);
        tree.insert(2);

        assert_eq!(tree.in_order_vec(), vec![0, 1, 2]);

        let root = tree.root.into_node();
        unsafe {
            assert_eq!((*root.as_ptr()).value, 1);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.into_node();
            assert_eq!((*left_child.as_ptr()).value, 0);
            assert_eq!((*left_child.as_ptr()).color, Color::Red);
            let right_child = (*root.as_ptr()).right.into_node();
            assert_eq!((*right_child.as_ptr()).value, 2);
            assert_eq!((*right_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn min_and_max_height() {
        use super::Link;
        unsafe {
            let nil_node = NilNode::new();
            let nil = Link::Nil(nil_node);
            let nodes = Node::new(0, nil, nil);
            let left = Link::Real(Node::new(0, Link::Real(nodes), nil));
            let right = Link::Real(Node::new(0, Link::Real(nodes), nil));
            (*nodes.as_ptr()).left = left;
            (*nodes.as_ptr()).right = right;

            let left_left = Link::Real(Node::new(0, left, nil));
            (*left.into_node().as_ptr()).left = left_left;
            let left_left_left = Link::Real(Node::new(0, left_left, nil));
            (*left_left.into_node().as_ptr()).left = left_left_left;

            assert_eq!(nodes.as_ref().height(), 4);
            assert_eq!(nodes.as_ref().min_height(), 2);

            Node::node_drop(nodes);

            let _ = Box::from_raw(nil_node.as_ptr());
        }
    }

    #[test]
    fn insertion_blackbox() {
        use rand::prelude::*;

        let mut tree = RBTree::<i32>::new();

        let mut rng = rand::rng();

        let items: Vec<i32> = (0..50).map(|_| rng.random_range(-100..100)).collect();

        for item in items {
            tree.insert(item);
        }

        let tree_vec = tree.in_order_vec();
        assert!(tree_vec.is_sorted());

        let max_height = unsafe { tree.root.into_node().as_ref().height() };
        let min_height = unsafe { tree.root.into_node().as_ref().min_height() };

        assert!((max_height as f32 / min_height as f32) < 2.0);
    }

    #[test]
    fn delete_root() {
        let mut tree = RBTree::new();

        tree.insert(0);
        tree.delete(&0);

        assert!(tree.root.is_nil());
    }

    #[test]
    fn delete_case_1_then_2_left() {
        let mut tree = RBTree::new();

        tree.insert(-7);
        tree.insert(-10);
        tree.insert(-4);
        tree.insert(-6);
        tree.insert(6);
        tree.insert(2);

        assert!(tree.delete(&-10));
        assert!(tree.in_order_vec().is_sorted());

        unsafe {
            let root = tree.root.into_node();
            assert_eq!((*root.as_ptr()).value, -4);
            assert_eq!((*root.as_ptr()).color, Color::Black);

            let left = (*root.as_ptr()).left.into_node();
            assert_eq!((*left.as_ptr()).value, -7);
            assert_eq!((*left.as_ptr()).color, Color::Black);

            let left_right = (*left.as_ptr()).right.into_node();
            assert_eq!((*left_right.as_ptr()).value, -6);
            assert_eq!((*left_right.as_ptr()).color, Color::Red);

            let right = (*root.as_ptr()).right.into_node();
            assert_eq!((*right.as_ptr()).value, 6);
            assert_eq!((*right.as_ptr()).color, Color::Black);

            let right_left = (*right.as_ptr()).left.into_node();
            assert_eq!((*right_left.as_ptr()).value, 2);
            assert_eq!((*right_left.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn delete_case_1_then_2_right() {
        let mut tree = RBTree::new();

        tree.insert(7);
        tree.insert(10);
        tree.insert(4);
        tree.insert(6);
        tree.insert(-6);
        tree.insert(-2);

        assert!(tree.delete(&10));
        assert!(tree.in_order_vec().is_sorted());

        unsafe {
            let root = tree.root.into_node();
            assert_eq!((*root.as_ptr()).value, 4);
            assert_eq!((*root.as_ptr()).color, Color::Black);

            let left = (*root.as_ptr()).left.into_node();
            assert_eq!((*left.as_ptr()).value, -6);
            assert_eq!((*left.as_ptr()).color, Color::Black);

            let left_right = (*left.as_ptr()).right.into_node();
            assert_eq!((*left_right.as_ptr()).value, -2);
            assert_eq!((*left_right.as_ptr()).color, Color::Red);

            let right = (*root.as_ptr()).right.into_node();
            assert_eq!((*right.as_ptr()).value, 7);
            assert_eq!((*right.as_ptr()).color, Color::Black);

            let right_left = (*right.as_ptr()).left.into_node();
            assert_eq!((*right_left.as_ptr()).value, 6);
            assert_eq!((*right_left.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn delete_case_3_then_4_left() {
        let mut tree = RBTree::new();

        let vals = vec![0, -10, 10, -12, 5, 3, 8];

        for val in vals {
            tree.insert(val);
        }

        assert!(tree.delete(&3));

        unsafe {
            let root = tree.root.into_node();
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Black);

            let left = (*root.as_ptr()).left.into_node();
            assert_eq!((*left.as_ptr()).value, -10);
            assert_eq!((*left.as_ptr()).color, Color::Black);

            let left_left = (*left.as_ptr()).left.into_node();
            assert_eq!((*left_left.as_ptr()).value, -12);
            assert_eq!((*left_left.as_ptr()).color, Color::Red);

            let right = (*root.as_ptr()).right.into_node();
            assert_eq!((*right.as_ptr()).value, 8);
            assert_eq!((*right.as_ptr()).color, Color::Red);

            let right_left = (*right.as_ptr()).left.into_node();
            assert_eq!((*right_left.as_ptr()).value, 5);
            assert_eq!((*right_left.as_ptr()).color, Color::Black);

            let right_right = (*right.as_ptr()).right.into_node();
            assert_eq!((*right_right.as_ptr()).value, 10);
            assert_eq!((*right_right.as_ptr()).color, Color::Black);
        }
    }

    #[test]
    fn delete_case_3_then_4_right() {
        let mut tree = RBTree::new();

        let vals = vec![0, 10, -10, 12, -5, -3, -8];

        for val in vals {
            tree.insert(val);
        }

        assert!(tree.delete(&-3));

        unsafe {
            let root = tree.root.into_node();
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Black);

            let left = (*root.as_ptr()).left.into_node();
            assert_eq!((*left.as_ptr()).value, -8);
            assert_eq!((*left.as_ptr()).color, Color::Red);

            let left_left = (*left.as_ptr()).left.into_node();
            assert_eq!((*left_left.as_ptr()).value, -10);
            assert_eq!((*left_left.as_ptr()).color, Color::Black);

            let left_right = (*left.as_ptr()).right.into_node();
            assert_eq!((*left_right.as_ptr()).value, -5);
            assert_eq!((*left_right.as_ptr()).color, Color::Black);

            let right = (*root.as_ptr()).right.into_node();
            assert_eq!((*right.as_ptr()).value, 10);
            assert_eq!((*right.as_ptr()).color, Color::Black);

            let right_right = (*right.as_ptr()).right.into_node();
            assert_eq!((*right_right.as_ptr()).value, 12);
            assert_eq!((*right_right.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn delete_case_4_left() {
        let mut tree = RBTree::new();

        let vals = vec![0, -10, 10, -12, 5, 3, 12];

        for val in vals {
            tree.insert(val);
        }

        assert!(tree.delete(&3));

        unsafe {
            let root = tree.root.into_node();
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Black);

            let left = (*root.as_ptr()).left.into_node();
            assert_eq!((*left.as_ptr()).value, -10);
            assert_eq!((*left.as_ptr()).color, Color::Black);

            let left_left = (*left.as_ptr()).left.into_node();
            assert_eq!((*left_left.as_ptr()).value, -12);
            assert_eq!((*left_left.as_ptr()).color, Color::Red);

            let right = (*root.as_ptr()).right.into_node();
            assert_eq!((*right.as_ptr()).value, 10);
            assert_eq!((*right.as_ptr()).color, Color::Red);

            let right_left = (*right.as_ptr()).left.into_node();
            assert_eq!((*right_left.as_ptr()).value, 5);
            assert_eq!((*right_left.as_ptr()).color, Color::Black);

            let right_right = (*right.as_ptr()).right.into_node();
            assert_eq!((*right_right.as_ptr()).value, 12);
            assert_eq!((*right_right.as_ptr()).color, Color::Black);
        }
    }

    #[test]
    fn delete_case_4_right() {
        let mut tree = RBTree::new();

        let vals = vec![0, 10, -10, 12, -5, -3, -12];

        for val in vals {
            tree.insert(val);
        }

        assert!(tree.delete(&-3));

        unsafe {
            let root = tree.root.into_node();
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Black);

            let left = (*root.as_ptr()).left.into_node();
            assert_eq!((*left.as_ptr()).value, -10);
            assert_eq!((*left.as_ptr()).color, Color::Red);

            let left_left = (*left.as_ptr()).left.into_node();
            assert_eq!((*left_left.as_ptr()).value, -12);
            assert_eq!((*left_left.as_ptr()).color, Color::Black);

            let left_right = (*left.as_ptr()).right.into_node();
            assert_eq!((*left_right.as_ptr()).value, -5);
            assert_eq!((*left_right.as_ptr()).color, Color::Black);

            let right = (*root.as_ptr()).right.into_node();
            assert_eq!((*right.as_ptr()).value, 10);
            assert_eq!((*right.as_ptr()).color, Color::Black);

            let right_right = (*right.as_ptr()).right.into_node();
            assert_eq!((*right_right.as_ptr()).value, 12);
            assert_eq!((*right_right.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn deletion_blackbox() {
        use rand::prelude::*;

        let mut tree = RBTree::<i32>::new();

        let mut rng = rand::rng();

        let items: Vec<i32> = (0..50).map(|_| rng.random_range(-100..100)).collect();

        for item in &items {
            tree.insert(*item);
        }

        let indices: Vec<usize> = (0..10).map(|_| rng.random_range(0..items.len())).collect();

        for index in indices {
            tree.delete(&items[index]);
        }

        let tree_vec = tree.in_order_vec();
        assert!(tree_vec.is_sorted());

        let max_height = unsafe { tree.root.into_node().as_ref().height() };
        let min_height = unsafe { tree.root.into_node().as_ref().min_height() };

        assert!((max_height as f32 / min_height as f32) < 2.0);
    }
}
