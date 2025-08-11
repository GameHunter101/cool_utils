use std::{
    collections::{HashMap, HashSet},
    ptr::NonNull,
};

use nalgebra::Vector2;

type Face = Vec<usize>;

struct DCEL {
    vertices: Vec<Vector2<f32>>,
    half_edges: HashMap<(usize, usize), NonNull<HalfEdge>>,
    faces: Vec<Face>,
}

impl DCEL {
    fn new(vertices: Vec<Vector2<f32>>, adjacency_list: HashMap<usize, HashSet<usize>>) -> Self {
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
                (*half_edge_ptr.as_ptr()).twin = half_edges[&(half_edge_id.1, half_edge_id.0)];
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

        let faces = Self::find_all_faces(&half_edges)
            .into_iter()
            .filter(|face| Self::signed_face_area(face, &vertices) >= 0.0).collect();

        Self {
            vertices,
            half_edges,
            faces,
        }
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

        unsorted_neighbors.sort_by(|&a, &b| {
            let a_vector = vertices[a] - vertices[vertex];
            let a_angle = a_vector.y.atan2(a_vector.x);

            let b_vector = vertices[b] - vertices[vertex];
            let b_angle = b_vector.y.atan2(b_vector.x);

            a_angle.total_cmp(&b_angle)
        });

        unsorted_neighbors
    }

    fn find_all_faces(half_edges: &HashMap<(usize, usize), NonNull<HalfEdge>>) -> Vec<Face> {
        let mut untraversed_half_edges: HashSet<(usize, usize)> =
            half_edges.keys().copied().collect();
        let mut faces = Vec::new();

        while let Some(half_edge_id) = untraversed_half_edges.iter().next().copied() {
            let bordering_half_edge_ids = Self::traverse_half_edge_loop(half_edge_id, half_edges);
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

    fn traverse_half_edge_loop(
        start: (usize, usize),
        half_edges: &HashMap<(usize, usize), NonNull<HalfEdge>>,
    ) -> Vec<(usize, usize)> {
        println!("Start: {start:?}");
        unsafe {
            let mut traversal_path = Vec::new();

            let mut current_half_edge = half_edges[&start];

            while let Some(next) = (*current_half_edge.as_ptr()).next {
                let next_origin = (*next.as_ptr()).origin;
                traversal_path.push(((*current_half_edge.as_ptr()).origin, next_origin));
                current_half_edge = next;
                println!("Next: {:?}", next.as_ref());
                if next_origin == start.0 {
                    break;
                }
            }

            traversal_path
        }
    }

    fn signed_face_area(face: &Face, vertices: &[Vector2<f32>]) -> f32 {
        0.5 * face[..face.len() - 1]
            .iter()
            .enumerate()
            .map(|(i, vertex_index)| {
                let current_vertex = vertices[*vertex_index];
                let next_vertex = vertices[face[i + 1]];

                current_vertex.x * next_vertex.y - next_vertex.x * current_vertex.y
            })
            .sum::<f32>()
    }

    fn faces(&self) -> &[Face] {
        &self.faces
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

impl Drop for HalfEdge {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.twin.as_ptr());
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet};

    use nalgebra::Vector2;

    use super::DCEL;
    type Point = Vector2<f32>;

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

        let dcel = DCEL::new(vertices, adjacency_list);

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
}
