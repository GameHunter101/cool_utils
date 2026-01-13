use std::fmt::Debug;

use nalgebra::Vector2;

#[derive(Debug)]
pub struct Quadtree<const N: usize> {
    center: Vector2<f32>,
    half_dimension: f32,
    points: [Vector2<f32>; N],
    num_points: usize,
    tr_node: Option<Box<Quadtree<N>>>,
    tl_node: Option<Box<Quadtree<N>>>,
    bl_node: Option<Box<Quadtree<N>>>,
    br_node: Option<Box<Quadtree<N>>>,
}

impl<const N: usize> Quadtree<N> {
    pub fn new(center: Vector2<f32>, half_dimension: f32) -> Self {
        Self {
            center,
            half_dimension,
            points: [Vector2::zeros(); N],
            num_points: 0,
            tr_node: None,
            tl_node: None,
            bl_node: None,
            br_node: None,
        }
    }

    pub fn insert(&mut self, point: Vector2<f32>) -> bool {
        if !(point.x > self.center.x - self.half_dimension
            && point.x < self.center.x + self.half_dimension)
            || !(point.y > self.center.y - self.half_dimension
                && point.y < self.center.y + self.half_dimension)
        {
            return false;
        }

        if self.num_points < N {
            self.points[self.num_points] = point;
            self.num_points += 1;
            true
        } else {
            if self.tr_node.is_none() {
                let (tr_node, tl_node, bl_node, br_node) = (
                    Quadtree::<N>::new(
                        self.center + Vector2::new(0.5, 0.5) * self.half_dimension,
                        self.half_dimension / 2.0,
                    ),
                    Quadtree::<N>::new(
                        self.center + Vector2::new(-0.5, 0.5) * self.half_dimension,
                        self.half_dimension / 2.0,
                    ),
                    Quadtree::<N>::new(
                        self.center - Vector2::new(0.5, 0.5) * self.half_dimension,
                        self.half_dimension / 2.0,
                    ),
                    Quadtree::<N>::new(
                        self.center + Vector2::new(0.5, -0.5) * self.half_dimension,
                        self.half_dimension / 2.0,
                    ),
                );
                self.tr_node = Some(Box::new(tr_node));
                self.tl_node = Some(Box::new(tl_node));
                self.bl_node = Some(Box::new(bl_node));
                self.br_node = Some(Box::new(br_node));
            }
            self.tr_node.as_mut().unwrap().insert(point)
                || self.tl_node.as_mut().unwrap().insert(point)
                || self.bl_node.as_mut().unwrap().insert(point)
                || self.br_node.as_mut().unwrap().insert(point)
        }
    }

    pub fn get_point_within_distance(
        &self,
        target: Vector2<f32>,
        distance: f32,
    ) -> Option<Vector2<f32>> {
        if let Some(valid_point) = self.points[..self.num_points]
            .iter()
            .filter(|point| point.metric_distance(&target) <= distance)
            .next()
        {
            Some(*valid_point)
        } else {
            let mut search_up = target.y >= self.center.y;
            let mut search_down = target.y <= self.center.y;
            let mut search_right = target.x >= self.center.x;
            let mut search_left = target.x <= self.center.x;

            if (target.x - self.center.x).abs() <= distance {
                search_right = true;
                search_left = true;
            }
            if (target.y - self.center.y).abs() <= distance {
                search_up = true;
                search_down = true;
            }

            if search_up {
                if search_right {
                    if let Some(tr_node) = self.tr_node.as_ref()
                        && let Some(tr_search) = tr_node.get_point_within_distance(target, distance)
                    {
                        return Some(tr_search);
                    }
                } else {
                    if let Some(tl_node) = self.tl_node.as_ref()
                        && let Some(tl_search) = tl_node.get_point_within_distance(target, distance)
                    {
                        return Some(tl_search);
                    }
                }
            } else {
                if search_right {
                    if let Some(br_node) = self.br_node.as_ref()
                        && let Some(br_search) = br_node.get_point_within_distance(target, distance)
                    {
                        return Some(br_search);
                    }
                } else {
                    if let Some(bl_node) = self.bl_node.as_ref()
                        && let Some(bl_search) = bl_node.get_point_within_distance(target, distance)
                    {
                        return Some(bl_search);
                    }
                }
            }

            None
        }
    }
}
