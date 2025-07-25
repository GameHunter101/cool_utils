use std::ptr::NonNull;

type Link<T> = Option<NonNull<Node<T>>>;

struct RBTree<T: Ord + std::fmt::Debug + Clone> {
    root: Link<T>,
}

impl<T: Ord + std::fmt::Debug + Clone> RBTree<T> {
    fn new() -> Self {
        Self { root: None }
    }

    fn insert(&mut self, element: T) {
        unsafe {
            if let Some(root) = self.root {
                Node::insert(root, element, &mut self.root);
            } else {
                let boxed_node = Node::new(element, None);

                self.root = Some(boxed_node);
            }
        }
    }

    fn delete(&mut self, element: T) -> bool {
        if let Some(root) = self.root {
            unsafe { Node::delete(root, element, &mut self.root) }
        } else {
            false
        }
    }

    fn in_order_vec(&self) -> Vec<T> {
        if let Some(root) = self.root {
            unsafe { root.as_ref().in_order_vec() }
        } else {
            Vec::new()
        }
    }
}

impl<T: Ord + std::fmt::Debug + Clone> Drop for RBTree<T> {
    fn drop(&mut self) {
        if let Some(root) = self.root {
            Node::node_drop(root);
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
enum Color {
    Red,
    Black,
}

struct Node<T: Ord + std::fmt::Debug> {
    value: T,
    color: Color,
    left: Link<T>,
    right: Link<T>,
    parent: Link<T>,
}

impl<T: Ord + std::fmt::Debug + Clone> Node<T> {
    fn new(element: T, parent: Link<T>) -> NonNull<Node<T>> {
        unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                value: element,
                color: Color::Red,
                left: None,
                right: None,
                parent,
            })))
        }
    }

    fn insert(node: NonNull<Node<T>>, element: T, root: &mut Link<T>) {
        unsafe {
            if element < (*node.as_ptr()).value {
                if let Some(left) = (*node.as_ptr()).left {
                    Self::insert(left, element, root);
                } else {
                    let boxed_node = Node::new(element, Some(node));
                    (*node.as_ptr()).left = Some(boxed_node);
                    Self::rebalance(boxed_node, root);
                }
            } else {
                if let Some(right) = (*node.as_ptr()).right {
                    Self::insert(right, element, root);
                } else {
                    let boxed_node = Node::new(element, Some(node));
                    (*node.as_ptr()).right = Some(boxed_node);
                    Self::rebalance(boxed_node, root);
                }
            }
        }
    }

    fn rebalance(mut node: NonNull<Node<T>>, root: &mut Link<T>) {
        unsafe {
            let potential_parent = (*node.as_ptr()).parent;
            if let Some(parent) = potential_parent {
                let parent_color = (*parent.as_ptr()).color;
                if parent_color == Color::Black {
                    // Case 1
                    println!("C1");
                    return;
                } else {
                    let potential_grandparent = (*parent.as_ptr()).parent;
                    if let Some(grandparent) = potential_grandparent {
                        let (potential_uncle, parent_is_left_of_grandparent) =
                            if (*grandparent.as_ptr()).left == (*node.as_ptr()).parent {
                                ((*grandparent.as_ptr()).right, true)
                            } else {
                                ((*grandparent.as_ptr()).left, false)
                            };

                        let uncle_color = if let Some(uncle) = potential_uncle {
                            (*uncle.as_ptr()).color
                        } else {
                            Color::Black
                        };

                        if parent_color == Color::Red && uncle_color == Color::Red {
                            // Case 2
                            println!("C2");
                            (*parent.as_ptr()).color = Color::Black;
                            (*potential_uncle.unwrap().as_ptr()).color = Color::Black;
                            (*grandparent.as_ptr()).color = Color::Red;
                            Self::rebalance(grandparent, root);
                        } else if parent_color == Color::Red && uncle_color == Color::Black {
                            let node_is_left_of_parent = (*parent.as_ptr()).left == Some(node);
                            if node_is_left_of_parent != parent_is_left_of_grandparent {
                                // Case 5
                                println!("C5");
                                if node_is_left_of_parent {
                                    Self::rotate_right(node);
                                } else {
                                    Self::rotate_left(node);
                                }
                            } else {
                                node = (*node.as_ptr()).parent.unwrap();
                            }
                            // case 6
                            println!("C6");
                            if parent_is_left_of_grandparent {
                                Self::rotate_right(node);
                            } else {
                                Self::rotate_left(node);
                            }
                            (*node.as_ptr()).color = Color::Black;
                            if parent_is_left_of_grandparent {
                                (*(*node.as_ptr()).right.unwrap().as_ptr()).color = Color::Red
                            } else {
                                (*(*node.as_ptr()).left.unwrap().as_ptr()).color = Color::Red
                            }
                            if (*node.as_ptr()).parent.is_none() {
                                *root = Some(node);
                            }
                        }
                    } else {
                        if parent_color == Color::Red {
                            println!("C4");
                            // Case 4
                            (*parent.as_ptr()).color = Color::Black;
                            return;
                        }
                    }
                }
            } else {
                println!("C3");
                // Case 3
                return;
            }
        }
    }

    fn rotate_left(node: NonNull<Node<T>>) {
        unsafe {
            let new_top = node;
            let new_left = (*node.as_ptr()).parent.unwrap();
            (*new_left.as_ptr()).right = (*new_top.as_ptr()).left;
            if let Some(right_child_of_new_left) = (*new_left.as_ptr()).right {
                (*right_child_of_new_left.as_ptr()).parent = Some(new_left);
            }
            (*new_top.as_ptr()).left = Some(new_left);
            (*new_top.as_ptr()).parent = (*new_left.as_ptr()).parent;
            (*new_left.as_ptr()).parent = Some(new_top);
            if let Some(new_parent) = (*new_top.as_ptr()).parent {
                let on_left_side_of_new_parent = (*new_parent.as_ptr()).left == Some(new_left);
                if on_left_side_of_new_parent {
                    (*new_parent.as_ptr()).left = Some(new_top);
                } else {
                    (*new_parent.as_ptr()).right = Some(new_top);
                }
            }
        }
    }

    fn rotate_right(node: NonNull<Node<T>>) {
        unsafe {
            let new_top = node;
            let new_right = (*node.as_ptr()).parent.unwrap();
            (*new_right.as_ptr()).left = (*new_top.as_ptr()).right;
            if let Some(left_child_of_new_right) = (*new_right.as_ptr()).left {
                (*left_child_of_new_right.as_ptr()).parent = Some(new_right);
            }
            (*new_top.as_ptr()).right = Some(new_right);
            (*new_top.as_ptr()).parent = (*new_right.as_ptr()).parent;
            (*new_right.as_ptr()).parent = Some(new_top);
            if let Some(new_parent) = (*new_top.as_ptr()).parent {
                let on_left_side_of_new_parent = (*new_parent.as_ptr()).left == Some(new_right);
                if on_left_side_of_new_parent {
                    (*new_parent.as_ptr()).left = Some(new_top);
                } else {
                    (*new_parent.as_ptr()).right = Some(new_top);
                }
            }
        }
    }

    fn node_drop(node: NonNull<Node<T>>) {
        unsafe {
            let boxed_node = Box::from_raw(node.as_ptr());

            if let Some(left) = boxed_node.left {
                Self::node_drop(left);
            }

            if let Some(right) = boxed_node.right {
                Self::node_drop(right);
            }
        }
    }

    fn print(&self, indent: &str, is_final: bool, append: &str) {
        unsafe {
            println!("{indent}+- {:?} # {:?} - {append}", self.value, self.color);
            if let Some(left) = self.left {
                let new_indent = format!("{indent}{}", if is_final { "   " } else { "|  " });
                left.as_ref().print(&new_indent, self.right.is_none(), "L");
            }

            if let Some(right) = self.right {
                let new_indent = format!("{indent}{}", if is_final { "   " } else { "|  " });
                right.as_ref().print(&new_indent, true, "R");
            }
        }
    }

    fn in_order_vec(&self) -> Vec<T> {
        unsafe {
            if self.left.is_none() && self.right.is_none() {
                return vec![self.value.clone()];
            }

            let left_vec = if let Some(left) = self.left {
                left.as_ref().in_order_vec()
            } else {
                Vec::new()
            };
            let right_vec = if let Some(right) = self.right {
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

    fn delete(node: NonNull<Node<T>>, element: T, root: &mut Link<T>) -> bool {
        unsafe {
            match element.cmp(&(*node.as_ptr()).value) {
                std::cmp::Ordering::Less => {
                    if let Some(left) = (*node.as_ptr()).left {
                        Self::delete(left, element, root)
                    } else {
                        false
                    }
                }
                std::cmp::Ordering::Equal => {
                    let child_count = (*node.as_ptr()).left.is_some() as i32
                        + (*node.as_ptr()).right.is_some() as i32;
                    if child_count == 2 {
                        // Simple case 1
                        let least_successor = Self::least_successor(node);
                        let boxed_least_successor = Box::from_raw(least_successor.as_ptr());
                        let least_successor_parent = boxed_least_successor.parent.unwrap();

                        (*least_successor_parent.as_ptr()).left = None;
                        (*least_successor_parent.as_ptr()).right = boxed_least_successor.right;
                        (*node.as_ptr()).value = boxed_least_successor.value;
                    } else if child_count == 1 {
                        // Simple case 2
                        let replaced_node = if let Some(left) = (*node.as_ptr()).left {
                            left
                        } else {
                            (*node.as_ptr()).right.unwrap()
                        };

                        if let Some(parent) = (*node.as_ptr()).parent {
                            if (*parent.as_ptr()).left == Some(node) {
                                (*parent.as_ptr()).left = Some(replaced_node);
                            } else {
                                (*parent.as_ptr()).right = Some(replaced_node);
                            }
                        } else {
                            (*replaced_node.as_ptr()).parent = None;
                            *root = Some(replaced_node);
                        }
                        let _ = Box::from_raw(node.as_ptr());
                    } else {
                        if (*node.as_ptr()).parent.is_none() {
                            // Simple case 3
                            let _ = Box::from_raw(node.as_ptr());
                            *root = None;
                        } else if (*node.as_ptr()).color == Color::Red {
                            // Simple case 4
                            let parent = (*node.as_ptr()).parent.unwrap();

                            if (*parent.as_ptr()).left == Some(node) {
                                (*parent.as_ptr()).left = None;
                            } else {
                                (*parent.as_ptr()).right = None;
                            }

                            let _ = Box::from_raw(node.as_ptr());
                        } else {
                            // Complex cases
                        }
                    }
                    true
                }
                std::cmp::Ordering::Greater => {
                    if let Some(right) = (*node.as_ptr()).right {
                        Self::delete(right, element, root)
                    } else {
                        false
                    }
                }
            }
        }
    }

    fn least_successor(node: NonNull<Node<T>>) -> NonNull<Node<T>> {
        unsafe { Self::follow_left((*node.as_ptr()).right.unwrap()) }
    }

    fn follow_left(node: NonNull<Node<T>>) -> NonNull<Node<T>> {
        unsafe {
            if let Some(left) = (*node.as_ptr()).left {
                Node::follow_left(left)
            } else {
                node
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use rand::distr::StandardUniform;

    use crate::data_structures::rbtree::Color;

    use super::RBTree;

    #[test]
    fn successfully_construct_empty_tree() {
        let tree: RBTree<i32> = RBTree::new();
        assert!(tree.root.is_none());
    }

    #[test]
    fn inserting_case_3_works_properly() {
        unsafe {
            let mut tree = RBTree::new();
            tree.insert(0);
            assert_eq!((*tree.root.unwrap().as_ptr()).value, 0);
            assert_eq!((*tree.root.unwrap().as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn inserting_case_4_works_properly() {
        unsafe {
            let mut tree = RBTree::new();
            tree.insert(0);
            tree.insert(1);

            let root = tree.root.unwrap();
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let child = (*root.as_ptr()).right.unwrap();
            assert_eq!((*child.as_ptr()).value, 1);
            assert_eq!((*child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn inserting_case_1_works_properly() {
        unsafe {
            let mut tree = RBTree::new();
            tree.insert(0);
            tree.insert(1);
            tree.insert(-1);

            let root = tree.root.unwrap();
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let child = (*root.as_ptr()).left.unwrap();
            assert_eq!((*child.as_ptr()).value, -1);
            assert_eq!((*child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn inserting_case_2_works_properly_no_recursion() {
        unsafe {
            let mut tree = RBTree::new();

            tree.insert(0);
            tree.insert(2);
            tree.insert(-2);

            tree.insert(-1);

            let root = tree.root.unwrap();
            assert_eq!((*root.as_ptr()).value, 0);
            assert_eq!((*root.as_ptr()).color, Color::Red);
            let left_child = (*root.as_ptr()).left.unwrap();
            let right_child = (*root.as_ptr()).right.unwrap();
            assert_eq!((*left_child.as_ptr()).value, -2);
            assert_eq!((*left_child.as_ptr()).color, Color::Black);
            assert_eq!((*right_child.as_ptr()).value, 2);
            assert_eq!((*right_child.as_ptr()).color, Color::Black);
            assert_eq!((*(*left_child.as_ptr()).right.unwrap().as_ptr()).value, -1);
            assert_eq!(
                (*(*left_child.as_ptr()).right.unwrap().as_ptr()).color,
                Color::Red
            );
        }
    }

    #[test]
    fn inserting_case_5_works_properly_on_left_combined_with_case_6() {
        unsafe {
            let mut tree = RBTree::new();

            tree.insert(0);
            tree.insert(-2);
            tree.insert(-1);

            let root = tree.root.unwrap();
            assert_eq!((*root.as_ptr()).value, -1);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.unwrap();
            let right_child = (*root.as_ptr()).right.unwrap();
            assert_eq!((*left_child.as_ptr()).value, -2);
            assert_eq!((*left_child.as_ptr()).color, Color::Red);
            assert_eq!((*right_child.as_ptr()).value, 0);
            assert_eq!((*right_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn inserting_case_5_works_properly_on_right_combined_with_case_6() {
        unsafe {
            let mut tree = RBTree::new();

            tree.insert(0);
            tree.insert(2);
            tree.insert(1);

            let root = tree.root.unwrap();
            assert_eq!((*root.as_ptr()).value, 1);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.unwrap();
            let right_child = (*root.as_ptr()).right.unwrap();
            assert_eq!((*left_child.as_ptr()).value, 0);
            assert_eq!((*left_child.as_ptr()).color, Color::Red);
            assert_eq!((*right_child.as_ptr()).value, 2);
            assert_eq!((*right_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn inserting_case_6_works_properly_on_left() {
        unsafe {
            let mut tree = RBTree::new();

            tree.insert(0);
            tree.insert(-1);
            tree.insert(-2);

            let root = tree.root.unwrap();
            assert_eq!((*root.as_ptr()).value, -1);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.unwrap();
            let right_child = (*root.as_ptr()).right.unwrap();
            assert_eq!((*left_child.as_ptr()).value, -2);
            assert_eq!((*left_child.as_ptr()).color, Color::Red);
            assert_eq!((*right_child.as_ptr()).value, 0);
            assert_eq!((*right_child.as_ptr()).color, Color::Red);
        }
    }

    #[test]
    fn inserting_case_6_works_properly_on_right() {
        unsafe {
            let mut tree = RBTree::new();

            tree.insert(0);
            tree.insert(1);
            tree.insert(2);

            let root = tree.root.unwrap();
            assert_eq!((*root.as_ptr()).value, 1);
            assert_eq!((*root.as_ptr()).color, Color::Black);
            let left_child = (*root.as_ptr()).left.unwrap();
            let right_child = (*root.as_ptr()).right.unwrap();
            assert_eq!((*left_child.as_ptr()).value, 0);
            assert_eq!((*left_child.as_ptr()).color, Color::Red);
            assert_eq!((*right_child.as_ptr()).value, 2);
            assert_eq!((*right_child.as_ptr()).color, Color::Red);
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
    }
}
