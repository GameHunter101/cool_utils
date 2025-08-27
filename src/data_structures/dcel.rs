use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    ptr::NonNull,
};

use nalgebra::{Matrix2, Matrix3, RowVector3, Vector2};

pub type Face = Vec<usize>;

pub struct DCEL {
    half_edges: Vec<HalfEdge>,
    faces: Vec<Face>,
}

impl DCEL {
    pub fn new(vertices: &[Vector2<f32>], adjacency_list: &HashMap<usize, HashSet<usize>>) -> Self {
        let mut remaining_half_edges_set: HashSet<HalfEdge> = vertices
            .iter()
            .enumerate()
            .flat_map(|(origin, vertex)| {
                adjacency_list[&origin]
                    .iter()
                    .map(|neighbor| HalfEdge::new(origin, *neighbor))
                    .collect::<Vec<_>>()
            })
            .collect();

        let mut inserted_half_edges_map: HashMap<(usize, usize), usize> =
            HashMap::with_capacity(remaining_half_edges_set.len());

        let mut half_edges: Vec<HalfEdge> = Vec::with_capacity(remaining_half_edges_set.len());

        for half_edge in remaining_half_edges_set.drain() {
            if !inserted_half_edges_map
                .contains_key(&(half_edge.origin_vert, half_edge.terminus_vert))
            {
                let base_index = half_edges.len();
                let twin_index = half_edges.len() + 1;

                let base_half_edge = HalfEdge {
                    twin: twin_index,
                    ..half_edge
                };
                half_edges.push(base_half_edge);
                inserted_half_edges_map
                    .insert((half_edge.origin_vert, half_edge.terminus_vert), base_index);
                let twin_half_edge = HalfEdge {
                    origin_vert: half_edge.terminus_vert,
                    terminus_vert: half_edge.origin_vert,
                    twin: base_index,
                    ..half_edge
                };
                half_edges.push(twin_half_edge);
                inserted_half_edges_map
                    .insert((half_edge.terminus_vert, half_edge.origin_vert), twin_index);
            }
        }

        Self::assign_next_indices(
            vertices,
            &mut half_edges,
            inserted_half_edges_map,
            &adjacency_list,
        );

        let faces = Self::find_all_faces(&mut half_edges)
            .into_iter()
            .filter(|face| {
                !face.is_empty()
                    && face[0] != usize::MAX
                    && Self::face_orientation(face, &vertices) >= 0.0
            })
            .collect();

        Self { half_edges, faces }
    }

    fn assign_next_indices(
        vertices: &[Vector2<f32>],
        half_edges: &mut [HalfEdge],
        half_edges_map: HashMap<(usize, usize), usize>,
        adjacency_list: &HashMap<usize, HashSet<usize>>,
    ) {
        for half_edge in half_edges {
            let sorted_neighbors = Self::sorted_vertex_neighbors(
                half_edge.terminus_vert,
                half_edge.origin_vert,
                vertices,
                adjacency_list,
            );

            let index_of_next_half_edge = if let Some(next_vertex_index) = sorted_neighbors.first()
            {
                half_edges_map[&(half_edge.terminus_vert, *next_vertex_index)]
            } else {
                half_edge.twin
            };

            half_edge.next = index_of_next_half_edge;
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

    fn find_all_faces(half_edges: &mut [HalfEdge]) -> Vec<Face> {
        let mut faces = vec![Vec::new()];

        for i in 0..half_edges.len() {
            let mut edge = &mut half_edges[i];
            let face_id = faces.len() - 1;
            while edge.face_id == usize::MAX {
                edge.face_id = face_id;
                faces[face_id].push(edge.origin_vert);
                edge = &mut half_edges[edge.next];
            }

            if !faces[face_id].is_empty() {
                faces.push(Vec::new());
            }
        }

        faces
    }

    fn face_orientation(face: &Face, vertices: &[Vector2<f32>]) -> f32 {
        if face.len() < 3 {
            return -1.0;
        }
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct HalfEdge {
    origin_vert: usize,
    terminus_vert: usize,
    twin: usize,
    next: usize,
    face_id: usize,
}

impl Hash for HalfEdge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.origin_vert.hash(state);
        self.terminus_vert.hash(state);
    }
}

impl HalfEdge {
    fn new(origin: usize, terminus: usize) -> Self {
        Self {
            origin_vert: origin,
            terminus_vert: terminus,
            twin: usize::MAX,
            next: usize::MAX,
            face_id: usize::MAX,
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

        let dcel = DCEL::new(&vertices, &adjacency_list);

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

        let dcel = DCEL::new(&vertices, &adjacency_list);

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

        let dcel = DCEL::new(&vertices, &adjacency_list);

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

        let dcel = DCEL::new(&vertices, &adjacency_list);

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

        let dcel = DCEL::new(&vertices, &adjacency_list);

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

        let dcel = DCEL::new(&vertices, &adjacency_list);

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

        let dcel = DCEL::new(&vertices, &adjacency_list);

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

        let dcel = DCEL::new(&vertices, &adjacency_list);

        assert_eq!(dcel.half_edges.len(), 14);
        dbg!(dcel.faces());
        assert!(dcel.faces().is_empty());
    }

    #[test]
    fn weird_triangle_is_detected_and_line_segment_is_not() {
        let vertices = vec![
            Point::new(346.66837, 251.22778),
            Point::new(352.5979, 229.02747),
            Point::new(354.24518, 202.9497),
            Point::new(355.0, 191.0),
            Point::new(344.70496, 200.32208),
            Point::new(366.77185, 206.3999),
            Point::new(385.24808, 210.80338),
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
        ];

        let adjacency_list: HashMap<usize, HashSet<usize>> = HashMap::from_iter(vec![
            (0, HashSet::from_iter(vec![1])),
            (1, HashSet::from_iter(vec![0, 2])),
            (2, HashSet::from_iter(vec![1, 3, 4, 5])),
            (3, HashSet::from_iter(vec![2, 4])),
            (4, HashSet::from_iter(vec![2, 3])),
            (5, HashSet::from_iter(vec![2, 6])),
            (6, HashSet::from_iter(vec![5])),
            (7, HashSet::from_iter(vec![8])),
            (8, HashSet::from_iter(vec![7])),
        ]);

        let dcel = DCEL::new(&vertices, &adjacency_list);

        assert_eq!(dcel.faces().len(), 1);
    }
}
