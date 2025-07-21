use std::cmp::Ordering;
use std::ptr::NonNull;

use rand::prelude::*;

type Link<T> = NonNull<Node<T>>;

#[derive(PartialEq, Debug, Clone)]
struct Node<T: Ord + std::fmt::Debug + Clone> {
    node_type: NodeType<T>,
    next_ptrs: Vec<Link<T>>,
    prev_ptrs: Vec<Link<T>>,
}

impl<T: Ord + Clone + std::fmt::Debug> Node<T> {
    fn new_empty_chain() -> NonNull<Node<T>> {
        unsafe {
            let mut end_node = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                node_type: NodeType::End,
                next_ptrs: Vec::new(),
                prev_ptrs: Vec::new(),
            })));

            let start_node = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                node_type: NodeType::Start,
                next_ptrs: vec![end_node],
                prev_ptrs: Vec::new(),
            })));

            (*end_node.as_ptr()).prev_ptrs = vec![start_node];

            start_node
        }
    }

    fn traverse_level(start: Link<T>, level: usize, element: T) -> (Link<T>, Vec<Link<T>>) {
        unsafe {
            if let Some(next) = start.as_ref().next_ptrs.get(level).copied() {
                if (*next.as_ptr()).node_type > NodeType::Value(element.clone()) {
                    if level == 0 {
                        (start, Vec::new())
                    } else {
                        let (node, mut path) = Self::traverse_level(start, level - 1, element);
                        path.push(start);
                        (node, path)
                    }
                } else {
                    Self::traverse_level(next, level, element)
                }
            } else {
                (start, Vec::new())
            }
        }
    }

    fn append(
        origin: Link<T>,
        end: Link<T>,
        node: Link<T>,
        element: T,
        traversal_path: Vec<Link<T>>,
        rng: &mut StdRng,
    ) {
        unsafe {
            let mut next_ptrs = Vec::new();
            let mut prev_ptrs = Vec::new();

            next_ptrs.push(node.as_ref().next_ptrs[0]);
            prev_ptrs.push(node);

            for (level, node) in traversal_path.iter().copied().enumerate() {
                let promotion: bool = rng.random();
                if promotion {
                    next_ptrs.push(node.as_ref().next_ptrs[level + 1]);
                    prev_ptrs.push(node);
                } else {
                    break;
                }
            }

            let mut new_node = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                node_type: NodeType::Value(element),
                next_ptrs: next_ptrs.clone(),
                prev_ptrs: prev_ptrs.clone(),
            })));

            for (level, node) in prev_ptrs.into_iter().enumerate() {
                (&mut (*node.as_ptr()).next_ptrs)[level] = new_node;
            }

            let promotion_count = next_ptrs.len();

            for (level, node) in next_ptrs.into_iter().enumerate() {
                (&mut (*node.as_ptr()).prev_ptrs)[level] = new_node;
            }

            if (traversal_path.len() == 0 || promotion_count == traversal_path.len())
                && rng.random()
            {
                (*origin.as_ptr()).next_ptrs.push(new_node);
                (*end.as_ptr()).prev_ptrs.push(new_node);

                (*new_node.as_ptr()).next_ptrs.push(end);
                (*new_node.as_ptr()).prev_ptrs.push(origin);
            }
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
enum NodeType<T: Ord + std::fmt::Debug + Clone> {
    Start,
    Value(T),
    End,
}

impl<T: Ord + std::fmt::Debug + Clone> Eq for NodeType<T> {}

impl<T: Ord + std::fmt::Debug + Clone> Ord for NodeType<T> {
    fn cmp(&self, other: &NodeType<T>) -> Ordering {
        match self {
            Self::Start => Ordering::Less,
            Self::Value(lhs) => match other {
                Self::Start => Ordering::Greater,
                Self::Value(rhs) => lhs.cmp(rhs),
                Self::End => Ordering::Less,
            },
            Self::End => Ordering::Greater,
        }
    }
}

struct SkipList<T: Ord + std::fmt::Debug + Clone> {
    nodes: Link<T>,
    end: Link<T>,
    rng: StdRng,
    len: usize,
}

impl<T: Ord + Clone + std::fmt::Debug> SkipList<T> {
    fn new(rng_seed: u64) -> Self {
        unsafe {
            let nodes = Node::new_empty_chain();
            Self {
                nodes,
                end: nodes.as_ref().next_ptrs[0],
                rng: StdRng::seed_from_u64(rng_seed),
                len: 0,
            }
        }
    }

    fn traverse(&self, element: T) -> (Link<T>, Vec<Link<T>>) {
        unsafe {
            Node::traverse_level(
                self.nodes,
                (*self.nodes.as_ptr()).next_ptrs.len() - 1,
                element.clone(),
            )
        }
    }

    fn height(&self) -> usize {
        unsafe { (*self.nodes.as_ptr()).next_ptrs.len() }
    }

    fn insert(&mut self, element: T) {
        unsafe {
            let (traverse_node, traverse_path) = Node::traverse_level(
                self.nodes,
                (*self.nodes.as_ptr()).next_ptrs.len() - 1,
                element.clone(),
            );

            Node::append(
                self.nodes,
                self.end,
                traverse_node,
                element,
                traverse_path,
                &mut self.rng,
            );

            self.len += 1;
        }
    }

    fn remove(&mut self, element: T) -> bool {
        unsafe {
            let (traverse_target, traverse_path) =
                Node::traverse_level(self.nodes, self.height() - 1, element.clone());

            /* let next_ptrs = std::iter::once(traverse_node.as_ref().next_ptrs[0])
            .into_iter().enumerate()
            .chain(traverse_path.iter().map(|node| node.)); */

            if (*traverse_target.as_ptr()).node_type == NodeType::Value(element) {
                let boxed_target = Box::from_raw(traverse_target.as_ptr());

                let node_prev_ptrs = boxed_target.prev_ptrs.clone();
                let node_next_ptrs = boxed_target.next_ptrs.clone();

                assert_eq!(node_prev_ptrs.len(), node_next_ptrs.len());

                for i in 0..node_next_ptrs.len() {
                    (&mut (*node_prev_ptrs[i].as_ptr()).next_ptrs)[i] = node_next_ptrs[i];
                    (&mut (*node_next_ptrs[i].as_ptr()).prev_ptrs)[i] = node_prev_ptrs[i];
                }

                self.len -= 1;

                true
            } else {
                false
            }
        }
    }

    fn iter(&self, level: usize) -> Iter<'_, T> {
        unsafe {
            Iter {
                next: if self.nodes.as_ref().next_ptrs.len() - 1 >= level {
                    Some(self.nodes.as_ref())
                } else {
                    None
                },
                level,
            }
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn len_at_level(&self, level: usize) -> usize {
        self.iter(level).count()
    }
}

impl<T: Ord + std::fmt::Debug + Clone> Drop for SkipList<T> {
    fn drop(&mut self) {
        unsafe {
            while !(*self.nodes.as_ptr()).next_ptrs.is_empty() {
                let boxed_node = Box::from_raw(self.nodes.as_ptr());
                self.nodes = boxed_node.next_ptrs[0];
            }
            let _ = Box::from_raw(self.nodes.as_ptr());
        }
    }
}

struct Iter<'a, T: Ord + std::fmt::Debug + Clone> {
    next: Option<&'a Node<T>>,
    level: usize,
}

impl<'a, T: Ord + std::fmt::Debug + Clone> Iterator for Iter<'a, T> {
    type Item = &'a NodeType<T>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.next.take().map(|node| {
                self.next = node.next_ptrs.get(self.level).map(|ptr| ptr.as_ref());
                &node.node_type
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Link, NodeType, SkipList};
    unsafe fn path_to_vec<T: Ord + Clone + std::fmt::Debug>(
        path: Vec<Link<T>>,
    ) -> Vec<NodeType<T>> {
        path.into_iter()
            .map(|node| (*node.as_ptr()).node_type.clone())
            .collect()
    }

    #[test]
    fn traverse_finds_proper_path_with_empty_list() {
        unsafe {
            let list: SkipList<i32> = SkipList::new(5);
            let (target, path) = list.traverse(4);

            assert_eq!(target.as_ref().node_type, NodeType::Start);
            assert_eq!(path_to_vec(path), Vec::new());
        }
    }

    #[test]
    fn traverse_finds_proper_path_with_non_empty_list() {
        unsafe {
            let mut list = SkipList::new(13);
            list.insert(3);
            assert_eq!(list.iter(1).count(), 3);
            let (target, path) = list.traverse(4);

            assert_eq!(target.as_ref().node_type, NodeType::Value(3));
            assert_eq!(path_to_vec(path), vec![NodeType::Value(3)]);
        }
    }

    #[test]
    fn traverse_finds_existing_node() {
        unsafe {
            let mut list = SkipList::new(79);
            list.insert(7);
            list.insert(6);
            let (traverse_res, _) = list.traverse(7);

            assert_eq!((*traverse_res.as_ptr()).node_type, NodeType::Value(7));
        }
    }

    #[test]
    fn insert_adds_first_node_properly() {
        let mut list: SkipList<i32> = SkipList::new(2);
        list.insert(4);
        assert_eq!(
            list.iter(0).cloned().collect::<Vec<_>>(),
            vec![NodeType::Start, NodeType::Value(4), NodeType::End]
        );
    }

    #[test]
    fn insert_adds_nodes_in_correct_order() {
        let mut list: SkipList<i32> = SkipList::new(2);
        list.insert(4);
        list.insert(5);
        list.insert(1);
        list.insert(12);
        assert_eq!(
            list.iter(0).cloned().collect::<Vec<_>>(),
            vec![
                NodeType::Start,
                NodeType::Value(1),
                NodeType::Value(4),
                NodeType::Value(5),
                NodeType::Value(12),
                NodeType::End
            ]
        );
    }

    #[test]
    fn remove_function_correctly_removes_single_element() {
        let mut list = SkipList::new(85);
        list.insert(8);
        assert_eq!(
            list.iter(0).cloned().collect::<Vec<_>>(),
            vec![NodeType::Start, NodeType::Value(8), NodeType::End]
        );
        assert!(list.remove(8));
        assert_eq!(list.len(), 0);
        assert_eq!(
            list.iter(0).cloned().collect::<Vec<_>>(),
            vec![NodeType::Start, NodeType::End]
        );
    }

    #[test]
    fn remove_function_does_nothing_when_target_does_not_exist() {
        let mut list = SkipList::new(95);
        list.insert(6);

        assert!(!list.remove(2));
        assert_eq!(list.len(), 1);
        assert_eq!(
            list.iter(0).cloned().collect::<Vec<_>>(),
            vec![NodeType::Start, NodeType::Value(6), NodeType::End]
        );
    }

    #[test]
    fn remove_function_correctly_removes_multiple_elements() {
        let mut list = SkipList::new(54);
        list.insert(6);
        list.insert(743);
        list.insert(9);
        list.insert(-12);
        list.insert(54);

        list.remove(9);
        list.remove(-12);
        list.remove(6);

        assert_eq!(list.len(), 2);
        assert_eq!(
            list.iter(0).cloned().collect::<Vec<_>>(),
            vec![
                NodeType::Start,
                NodeType::Value(54),
                NodeType::Value(743),
                NodeType::End
            ]
        );
    }
}
