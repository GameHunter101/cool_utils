use std::{
    collections::{HashMap, HashSet},
    ptr::NonNull,
};

use nalgebra::{Matrix2, Matrix3, RowVector3, Vector2};

pub type Face = Vec<usize>;

pub struct DCEL {
    vertices: Vec<Vector2<f32>>,
    half_edges: HashMap<(usize, usize), NonNull<HalfEdge>>,
    faces: Vec<Face>,
}

impl DCEL {
    pub fn new(
        vertices: Vec<Vector2<f32>>,
        adjacency_list: HashMap<usize, HashSet<usize>>,
        max_face_vertex_count: usize,
    ) -> Self {
        let half_edges: HashMap<(usize, usize), NonNull<HalfEdge>> = vertices
            .iter()
            .enumerate()
            .flat_map(|(origin, vertex)| {
                adjacency_list[&origin]
                    .iter()
                    .map(|neighbor| ((origin, *neighbor), HalfEdge::new(origin)))
                    .collect::<Vec<_>>()
            })
            .collect();

        for (half_edge_id, half_edge_ptr) in &half_edges {
            unsafe {
                (*half_edge_ptr.as_ptr()).twin = *half_edges
                    .get(&(half_edge_id.1, half_edge_id.0))
                    .expect(&format!(
                        "Failed to find half edge '{:?}' as twin for '{half_edge_id:?}",
                        (half_edge_id.1, half_edge_id.0)
                    ));
                let sorted_neighbors = Self::sorted_vertex_neighbors(
                    half_edge_id.1,
                    half_edge_id.0,
                    &vertices,
                    &adjacency_list,
                );

                let next_ptr = if let Some(next_vertex_index) = sorted_neighbors.get(0) {
                    half_edges[&(half_edge_id.1, *next_vertex_index)]
                } else {
                    (*half_edge_ptr.as_ptr()).twin
                };

                (*half_edge_ptr.as_ptr()).next = Some(next_ptr);
            }
        }

        let faces = Self::find_all_faces(&half_edges, max_face_vertex_count)
            .into_iter()
            .filter(|face| !face.is_empty() && Self::face_orientation(face, &vertices) >= 0.0)
            .collect();

        Self {
            vertices,
            half_edges,
            faces,
        }
    }

    fn cross_product_2d(u: Vector2<f32>, v: Vector2<f32>) -> f32 {
        u.x * v.y - u.y * v.x
    }

    fn sorted_vertex_neighbors(
        vertex: usize,
        origin: usize,
        vertices: &[Vector2<f32>],
        adjacency_list: &HashMap<usize, HashSet<usize>>,
    ) -> Vec<usize> {
        let mut unsorted_neighbors: Vec<usize> = adjacency_list[&vertex]
            .iter()
            .filter(|&index| index != &origin)
            .copied()
            .collect();

        let current_direction = vertices[vertex] - vertices[origin];
        unsorted_neighbors.sort_by(|&a, &b| {
            let a_direction = vertices[a] - vertices[vertex];

            let b_direction = vertices[b] - vertices[vertex];

            let a_angle_unsigned = a_direction.angle(&current_direction);
            let a_angle_sign = Self::cross_product_2d(current_direction, a_direction).signum();
            let a_angle = a_angle_unsigned * a_angle_sign;

            let b_angle_unsigned = b_direction.angle(&current_direction);
            let b_angle_sign = Self::cross_product_2d(current_direction, b_direction).signum();
            let b_angle = b_angle_unsigned * b_angle_sign;

            a_angle.total_cmp(&b_angle).reverse()
        });

        unsorted_neighbors
    }

    fn find_all_faces(
        half_edges: &HashMap<(usize, usize), NonNull<HalfEdge>>,
        max_face_vertex_count: usize,
    ) -> Vec<Face> {
        let mut untraversed_half_edges: HashSet<(usize, usize)> =
            half_edges.keys().copied().collect();
        let mut faces = Vec::new();

        while let Some(half_edge_id) =
            Self::get_next_valid_loop_start(&untraversed_half_edges, half_edges)
        {
            let bordering_half_edge_ids =
                Self::traverse_half_edge_loop(half_edge_id, half_edges, max_face_vertex_count);
            untraversed_half_edges = untraversed_half_edges
                .difference(&HashSet::from_iter(bordering_half_edge_ids.clone()))
                .copied()
                .collect();
            faces.push(
                bordering_half_edge_ids
                    .into_iter()
                    .map(|(vertex_index, _)| vertex_index)
                    .collect::<Vec<_>>(),
            );
        }

        faces
    }

    fn get_next_valid_loop_start(
        untraversed_half_edges: &HashSet<(usize, usize)>,
        half_edges: &HashMap<(usize, usize), NonNull<HalfEdge>>,
    ) -> Option<(usize, usize)> {
        untraversed_half_edges
            .iter()
            .filter(|&half_edge_index| unsafe {
                (*(*(*half_edges[half_edge_index].as_ptr())
                    .next
                    .unwrap()
                    .as_ptr())
                .twin
                .as_ptr())
                .origin
                    != half_edge_index.0
            })
            .next()
            .copied()
    }

    fn traverse_half_edge_loop(
        start: (usize, usize),
        half_edges: &HashMap<(usize, usize), NonNull<HalfEdge>>,
        max_face_vertex_count: usize,
    ) -> Vec<(usize, usize)> {
        unsafe {
            let mut traversal_path = Vec::new();

            let mut current_half_edge = half_edges[&start];

            while let Some(next) = (*current_half_edge.as_ptr()).next {
                let next_origin = (*next.as_ptr()).origin;
                let next_terminus = (*(*next.as_ptr()).twin.as_ptr()).origin;
                traversal_path.push(((*current_half_edge.as_ptr()).origin, next_origin));
                current_half_edge = next;
                if (next_origin, next_terminus) == start {
                    break;
                }
                if traversal_path.len() > max_face_vertex_count {
                    return Vec::new();
                }
            }

            traversal_path
        }
    }

    fn face_orientation(face: &Face, vertices: &[Vector2<f32>]) -> f32 {
        assert!(face.len() > 2);
        let most_suitable_index = face.iter().fold(face[0], |acc, index| {
            match vertices[*index].x.total_cmp(&vertices[acc].x) {
                std::cmp::Ordering::Less => *index,
                std::cmp::Ordering::Equal => {
                    if vertices[*index].y < vertices[acc].y {
                        *index
                    } else {
                        acc
                    }
                }
                std::cmp::Ordering::Greater => acc,
            }
        });
        let mut index_of_most_suitable_in_face = face
            .iter()
            .position(|idx| idx == &most_suitable_index)
            .unwrap() as i32;

        let mut left_neighbor_index =
            face[((index_of_most_suitable_in_face - 1).rem_euclid(face.len() as i32)) as usize];
        let mut right_neighbor_index =
            face[((index_of_most_suitable_in_face + 1) % face.len() as i32) as usize];

        let mut deduped_indices: HashSet<usize> = HashSet::from_iter(face.clone());

        if left_neighbor_index == right_neighbor_index
            || face.len() == (deduped_indices.len() - 1) * 2
        {
            -1.0
        } else {
            Self::cross_product_2d(
                vertices[most_suitable_index] - vertices[left_neighbor_index],
                vertices[right_neighbor_index] - vertices[most_suitable_index],
            )
        }
    }

    pub fn faces(&self) -> &[Face] {
        &self.faces
    }
}

impl Drop for DCEL {
    fn drop(&mut self) {
        self.half_edges.values().for_each(|ptr| unsafe {
            Box::from_raw(ptr.as_ptr());
        });
    }
}

type Link = Option<NonNull<HalfEdge>>;

#[derive(Debug)]
struct HalfEdge {
    origin: usize,
    twin: NonNull<HalfEdge>,
    next: Link,
}

impl HalfEdge {
    fn new(origin: usize) -> NonNull<Self> {
        unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Self {
                origin,
                twin: NonNull::dangling(),
                next: None,
            })))
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet};

    use nalgebra::Vector2;

    use super::{DCEL, Face};
    type Point = Vector2<f32>;

    fn same_faces(real_faces: &[Face], expected_faces: Vec<Face>) -> bool {
        real_faces.len() == expected_faces.len()
            && expected_faces.iter().all(|face| {
                let expected_face_hash_set: HashSet<usize> = HashSet::from_iter(face.clone());
                real_faces.iter().any(|real_face| {
                    real_face.len() == face.len()
                        && HashSet::from_iter(real_face.clone())
                            .difference(&expected_face_hash_set)
                            .count()
                            == 0
                })
            })
    }

    #[test]
    fn simple_triangle_test_detects_face() {
        let vertices = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(2.0, 0.0),
        ];

        let adjacency_list: HashMap<usize, HashSet<usize>> = HashMap::from_iter(vec![
            (0, HashSet::from_iter(vec![1, 2])),
            (1, HashSet::from_iter(vec![0, 2])),
            (2, HashSet::from_iter(vec![0, 1])),
        ]);

        let dcel = DCEL::new(vertices, adjacency_list, 100);

        assert_eq!(dcel.half_edges.len(), 6);
        assert_eq!(dcel.faces().len(), 1);
        assert_eq!(dcel.faces()[0].len(), 3);
        assert_eq!(
            HashSet::<usize>::from_iter(dcel.faces()[0].to_vec())
                .difference(&HashSet::from_iter([0, 1, 2]))
                .count(),
            0
        );
    }

    #[test]
    fn multiple_simple_faces_detected() {
        let vertices = vec![
            Point::new(2.0, 2.0),
            Point::new(3.5, 0.0),
            Point::new(3.0, 3.5),
            Point::new(0.0, 3.0),
            Point::new(1.0, 4.0),
            Point::new(2.0, 5.0),
            Point::new(4.0, 4.0),
        ];

        let adjacency_list: HashMap<usize, HashSet<usize>> = HashMap::from_iter(vec![
            (0, HashSet::from_iter(vec![1, 2, 3])),
            (1, HashSet::from_iter(vec![0, 3, 6])),
            (2, HashSet::from_iter(vec![0, 5, 6])),
            (3, HashSet::from_iter(vec![0, 1, 4])),
            (4, HashSet::from_iter(vec![3, 5])),
            (5, HashSet::from_iter(vec![2, 4, 6])),
            (6, HashSet::from_iter(vec![1, 2, 5])),
        ]);

        let dcel = DCEL::new(vertices, adjacency_list, 100);

        assert_eq!(dcel.half_edges.len(), 20);

        let expected_faces = vec![
            vec![0, 1, 3],
            vec![0, 1, 2, 6],
            vec![0, 2, 3, 4, 5],
            vec![2, 5, 6],
        ];

        assert!(same_faces(dcel.faces(), expected_faces));
    }

    #[test]
    fn multiple_unconnected_faces() {
        let vertices = vec![
            Point::new(2.0, 0.0),
            Point::new(2.0, 2.0),
            Point::new(2.4, 3.0),
            Point::new(4.0, 2.0),
            Point::new(5.0, 4.0),
            Point::new(6.0, 2.0),
            Point::new(8.0, 2.0),
            Point::new(7.0, 3.0),
            Point::new(8.0, 5.0),
        ];

        let adjacency_list: HashMap<usize, HashSet<usize>> = HashMap::from_iter(vec![
            (0, HashSet::from_iter(vec![1, 3])),
            (1, HashSet::from_iter(vec![0, 2, 3])),
            (2, HashSet::from_iter(vec![1, 3])),
            (3, HashSet::from_iter(vec![2, 1, 0])),
            (4, HashSet::from_iter(vec![5, 8])),
            (5, HashSet::from_iter(vec![4, 6, 7])),
            (6, HashSet::from_iter(vec![5, 7, 8])),
            (7, HashSet::from_iter(vec![6, 5])),
            (8, HashSet::from_iter(vec![4, 6])),
        ]);

        let dcel = DCEL::new(vertices, adjacency_list, 100);

        assert_eq!(dcel.half_edges.len(), 22);

        let expected_faces = vec![
            vec![0, 1, 3],
            vec![1, 2, 3],
            vec![5, 6, 7],
            vec![4, 5, 6, 7, 8],
        ];

        assert!(same_faces(dcel.faces(), expected_faces));
    }

    #[test]
    fn degenerate_edge_still_detects_face() {
        let vertices = vec![
            Point::new(2.0, 0.0),
            Point::new(2.0, 2.0),
            Point::new(2.4, 3.0),
            Point::new(4.0, 2.0),
            Point::new(2.5, 2.0),
            Point::new(0.5, 1.8),
        ];

        let adjacency_list: HashMap<usize, HashSet<usize>> = HashMap::from_iter(vec![
            (0, HashSet::from_iter(vec![1, 3, 5])),
            (1, HashSet::from_iter(vec![0, 2, 4])),
            (2, HashSet::from_iter(vec![1, 3, 5])),
            (3, HashSet::from_iter(vec![2, 0])),
            (4, HashSet::from_iter(vec![1])),
            (5, HashSet::from_iter(vec![0, 2])),
        ]);

        let dcel = DCEL::new(vertices, adjacency_list, 100);

        assert_eq!(dcel.half_edges.len(), 14);

        let expected_faces = vec![vec![0, 3, 2, 1, 4, 1], vec![0, 5, 2, 1]];

        assert!(same_faces(dcel.faces(), expected_faces));
    }

    #[test]
    fn degenerate_edge_still_detects_face_2() {
        let vertices = vec![
            Point::new(2.0, 0.0),
            Point::new(2.0, 2.0),
            Point::new(2.4, 3.0),
            Point::new(4.0, 2.0),
            Point::new(2.5, 2.0),
        ];

        let adjacency_list: HashMap<usize, HashSet<usize>> = HashMap::from_iter(vec![
            (0, HashSet::from_iter(vec![1, 3])),
            (1, HashSet::from_iter(vec![0, 2, 4])),
            (2, HashSet::from_iter(vec![1, 3])),
            (3, HashSet::from_iter(vec![2, 0])),
            (4, HashSet::from_iter(vec![1])),
        ]);

        let dcel = DCEL::new(vertices, adjacency_list, 100);

        assert_eq!(dcel.half_edges.len(), 10);

        let expected_faces = vec![vec![0, 3, 2, 1, 4, 1]];

        assert!(same_faces(dcel.faces(), expected_faces));
    }

    #[test]
    fn skip_over_unconnected_face() {
        let vertices = vec![
            Point::new(2.0, 0.0),
            Point::new(2.0, 2.0),
            Point::new(2.4, 3.0),
            Point::new(4.0, 2.0),
            Point::new(2.5, 2.0),
            Point::new(0.5, 1.8),
        ];

        let adjacency_list: HashMap<usize, HashSet<usize>> = HashMap::from_iter(vec![
            (0, HashSet::from_iter(vec![1, 3, 5])),
            (1, HashSet::from_iter(vec![0, 2, 4])),
            (2, HashSet::from_iter(vec![1, 3])),
            (3, HashSet::from_iter(vec![2, 0])),
            (4, HashSet::from_iter(vec![1])),
            (5, HashSet::from_iter(vec![0])),
        ]);

        let dcel = DCEL::new(vertices, adjacency_list, 100);

        assert_eq!(dcel.half_edges.len(), 12);

        let expected_faces = vec![vec![0, 3, 2, 1, 4, 1]];

        assert!(same_faces(dcel.faces(), expected_faces));
    }

    #[test]
    fn detects_no_connected_faces() {
        let vertices = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(0.0, 1.0),
            Point::new(1.0, 3.0),
        ];

        let adjacency_list: HashMap<usize, HashSet<usize>> = HashMap::from_iter(vec![
            (0, HashSet::from_iter(vec![1])),
            (1, HashSet::from_iter(vec![0, 2])),
            (2, HashSet::from_iter(vec![1, 3])),
            (3, HashSet::from_iter(vec![2])),
            (4, HashSet::from_iter(Vec::new())),
        ]);

        let dcel = DCEL::new(vertices, adjacency_list, 100);

        assert_eq!(dcel.half_edges.len(), 6);

        assert!(dcel.faces().is_empty());
    }

    #[test]
    fn does_not_detect_face_in_line() {
        let vertices = vec![
            Point::new(200.40271, 239.94502),
            Point::new(200.54037, 230.66301),
            Point::new(200.47429, 222.01729),
            Point::new(182.41234, 206.6766),
            Point::new(176.8716, 187.22014),
            Point::new(184.08466, 169.4879),
            Point::new(189.91133, 158.69125),
            Point::new(193.0, 146.0),
        ];

        let adjacency_list: HashMap<usize, HashSet<usize>> = HashMap::from_iter(vec![
            (0, HashSet::from_iter(vec![1])),
            (1, HashSet::from_iter(vec![0, 2])),
            (2, HashSet::from_iter(vec![1, 3])),
            (3, HashSet::from_iter(vec![2, 4])),
            (4, HashSet::from_iter(vec![3, 5])),
            (5, HashSet::from_iter(vec![4, 6])),
            (6, HashSet::from_iter(vec![5, 7])),
            (7, HashSet::from_iter(vec![6])),
        ]);

        let dcel = DCEL::new(vertices, adjacency_list, 100);

        assert_eq!(dcel.half_edges.len(), 14);
        dbg!(dcel.faces());
        assert!(dcel.faces().is_empty());
    }
}
