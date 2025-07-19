use std::cmp::Ordering;
use std::ptr::NonNull;

use rand::prelude::*;

type Link<T> = Option<NonNull<Node<T>>>;

#[derive(PartialEq, Debug)]
struct Node<T: Ord + std::fmt::Debug> {
    node_type: NodeType<T>,
    next: Link<T>,
    upper_next: Link<T>,
    prev: Link<T>,
    upper_prev: Link<T>,
}

impl<T: Ord + Clone + std::fmt::Debug> Node<T> {
    fn new_empty_chain() -> NonNull<Node<T>> {
        unsafe {
            let mut end_node = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                node_type: NodeType::End,
                next: None,
                upper_next: None,
                prev: None,
                upper_prev: None,
            })));

            let start_node = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                node_type: NodeType::Start,
                next: Some(end_node),
                upper_next: Some(end_node),
                prev: None,
                upper_prev: None,
            })));

            (*end_node.as_ptr()).prev = Some(start_node);
            (*end_node.as_ptr()).upper_prev = Some(start_node);

            start_node
        }
    }

    fn traverse_upper(start: NonNull<Node<T>>, element: T) -> NonNull<Node<T>> {
        unsafe {
            if let Some(upper_next) = (*start.as_ptr()).upper_next {
                if (*upper_next.as_ptr()).node_type > NodeType::Value(element.clone()) {
                    start
                } else {
                    Self::traverse_upper(upper_next, element)
                }
            } else {
                start
            }
        }
    }

    fn traverse_lower(start: NonNull<Node<T>>, element: T) -> NonNull<Node<T>> {
        unsafe {
            if let Some(next) = (*start.as_ptr()).next {
                if (*next.as_ptr()).node_type > NodeType::Value(element.clone()) {
                    start
                } else {
                    Self::traverse_lower(next, element)
                }
            } else {
                start
            }
        }
    }

    fn append(
        node: NonNull<Node<T>>,
        element: T,
        previous_upper: NonNull<Node<T>>,
        rng: &mut StdRng,
    ) {
        unsafe {
            let promotion: bool = rng.random();
            let new_node = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                node_type: NodeType::Value(element),
                next: (*node.as_ptr()).next,
                upper_next: if promotion {
                    (*previous_upper.as_ptr()).upper_next
                } else {
                    None
                },
                prev: Some(node),
                upper_prev: if promotion {
                    Some(previous_upper)
                } else {
                    None
                }
            })));

            (*node.as_ptr()).next = Some(new_node);
            (*(*new_node.as_ptr()).next.unwrap().as_ptr()).prev = Some(new_node);
            if promotion {
                (*previous_upper.as_ptr()).upper_next = Some(new_node);
                (*(*new_node.as_ptr()).upper_next.unwrap().as_ptr()).upper_prev = Some(new_node);
            }
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
enum NodeType<T: Ord + std::fmt::Debug> {
    Start,
    Value(T),
    End,
}

impl<T: Ord + std::fmt::Debug> Eq for NodeType<T> {}

impl<T: Ord + std::fmt::Debug> Ord for NodeType<T> {
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

struct SkipList<T: Ord + std::fmt::Debug> {
    floor_level: NonNull<Node<T>>,
    upper_level: NonNull<Node<T>>,
    rng: StdRng,
}

impl<T: Ord + Clone + std::fmt::Debug> SkipList<T> {
    fn new(rng_seed: u64,) -> Self {
        let floor_level = Node::new_empty_chain();
        Self {
            floor_level,
            upper_level: floor_level,
            rng: StdRng::seed_from_u64(rng_seed)
        }
    }

    fn insert(&mut self, element: T) {
        unsafe {
            let upper_node = Node::traverse_upper(self.upper_level, element.clone());
            let lower_node = Node::traverse_lower(upper_node, element.clone());

            Node::append(lower_node, element, upper_node, &mut self.rng);
        }
    }

    fn remove(&mut self, element: T) -> bool {
        unsafe {
            let upper_node = Node::traverse_upper(self.upper_level, element.clone());
            if (*upper_node.as_ptr()).node_type == NodeType::Value(element.clone()) {
                let boxed_node = Box::from_raw(upper_node.as_ptr());
                (*boxed_node.prev.unwrap().as_ptr()).next = boxed_node.next;
                (*boxed_node.upper_prev.unwrap().as_ptr()).upper_next = boxed_node.upper_next;
                (*boxed_node.next.unwrap().as_ptr()).prev = boxed_node.prev;
                (*boxed_node.upper_next.unwrap().as_ptr()).upper_prev = boxed_node.upper_prev;
                return true;
            }
            let lower_node = Node::traverse_lower(upper_node, element.clone());
            if (*lower_node.as_ptr()).node_type == NodeType::Value(element.clone()) {
                let boxed_node = Box::from_raw(lower_node.as_ptr());
                (*boxed_node.prev.unwrap().as_ptr()).next = boxed_node.next;
                (*boxed_node.next.unwrap().as_ptr()).prev = boxed_node.prev;
                true
            } else {
                false
            }
        }
    }

    fn lower_iter(&self) -> LowerIter<'_, T> {
        unsafe {
            LowerIter {
                next: Some(self.floor_level.as_ref()),
            }
        }
    }

    fn upper_iter(&self) -> UpperIter<'_, T> {
        unsafe {
            UpperIter {
                upper_next: Some(self.upper_level.as_ref()),
            }
        }
    }
}

impl<T: Ord + std::fmt::Debug> Drop for SkipList<T> {
    fn drop(&mut self) {
        unsafe {
            while let Some(next) = (*self.floor_level.as_ptr()).next {
                let boxed_node = Box::from_raw(next.as_ptr());
                (*self.floor_level.as_ptr()).next = boxed_node.next;
            }
            let _ = Box::from_raw(self.floor_level.as_ptr());
        }
    }
}

struct LowerIter<'a, T: Ord + std::fmt::Debug> {
    next: Option<&'a Node<T>>,
}

impl<'a, T: Ord + std::fmt::Debug> Iterator for LowerIter<'a, T> {
    type Item = &'a NodeType<T>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.next.take().map(|node| {
                self.next = node.next.map(|ptr| ptr.as_ref());
                &node.node_type
            })
        }
    }
}

struct UpperIter<'a, T: Ord + std::fmt::Debug> {
    upper_next: Option<&'a Node<T>>,
}

impl<'a, T: Ord + std::fmt::Debug> Iterator for UpperIter<'a, T> {
    type Item = &'a NodeType<T>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.upper_next.take().map(|node| {
                self.upper_next = node.upper_next.map(|ptr| ptr.as_ref());
                &node.node_type
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::SkipList;
    #[test]
    fn basic() {
        let mut list: SkipList<i32> = SkipList::new(123);
        list.insert(4);
        println!("\nUpper: {:?}", list.upper_iter().collect::<Vec<_>>());
        println!("Lower: {:?}", list.lower_iter().collect::<Vec<_>>());
        list.insert(7);
        list.insert(1);
        list.insert(12);
        list.insert(30);
        println!("Upper: {:?}", list.upper_iter().collect::<Vec<_>>());
        println!("Lower: {:?}", list.lower_iter().collect::<Vec<_>>());
        list.remove(12);
        println!("Upper: {:?}", list.upper_iter().collect::<Vec<_>>());
        println!("Lower: {:?}", list.lower_iter().collect::<Vec<_>>());
        list.remove(4);
        println!("Upper: {:?}", list.upper_iter().collect::<Vec<_>>());
        println!("Lower: {:?}", list.lower_iter().collect::<Vec<_>>());
    }
}
