use bevy::math::Vec2;
use std::fmt::Debug;
use std::mem;

#[derive(Copy, Clone, Debug)]
pub struct Bounds {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

impl Bounds {
    pub fn overlaps(&self, other: Bounds) -> bool {
        (self.x_min <= other.x_max && self.x_max >= other.x_min)
            && (self.y_min <= other.y_max && self.y_max >= other.y_min)
    }

    pub fn contains<P: Point>(&self, point: P) -> bool {
        let point = point.xy();
        self.x_min < point[0]
            && self.x_max > point[0]
            && self.y_min < point[1]
            && self.y_max > point[1]
    }
}

pub trait Point: Copy {
    fn xy(&self) -> &[f32; 2];
}

impl Point for [f32; 2] {
    fn xy(&self) -> &[f32; 2] {
        self
    }
}

#[derive(Debug)]
pub struct QuadTree<UserData: Debug, const MAX_LEAF_ITEMS: usize> {
    node_data: NodeData<UserData, MAX_LEAF_ITEMS>,
    bounds: Bounds,
}

#[derive(Debug)]
pub enum NodeData<UserData: Debug, const MAX_LEAF_ITEMS: usize> {
    Branch {
        top_left: Box<QuadTree<UserData, MAX_LEAF_ITEMS>>,
        top_right: Box<QuadTree<UserData, MAX_LEAF_ITEMS>>,
        bottom_left: Box<QuadTree<UserData, MAX_LEAF_ITEMS>>,
        bottom_right: Box<QuadTree<UserData, MAX_LEAF_ITEMS>>,
    },
    Leaf(Vec<([f32; 2], UserData)>),
}

impl<UserData: Debug, const MAX_LEAF_ITEMS: usize> NodeData<UserData, MAX_LEAF_ITEMS> {
    pub fn child_nodes_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut Box<QuadTree<UserData, MAX_LEAF_ITEMS>>> {
        match self {
            NodeData::Branch {
                top_left,
                top_right,
                bottom_left,
                bottom_right,
            } => vec![top_left, top_right, bottom_left, bottom_right].into_iter(),
            NodeData::Leaf(_) => Vec::new().into_iter(),
        }
    }

    fn empty_leaf() -> Self {
        Self::Leaf(Vec::new())
    }
}

impl<UserData: Debug, const MAX_LEAF_ITEMS: usize> QuadTree<UserData, MAX_LEAF_ITEMS> {
    pub fn new(bounds: Bounds) -> Self {
        Self {
            node_data: NodeData::empty_leaf(),
            bounds,
        }
    }

    pub fn insert<P: Point>(&mut self, point: P, data: UserData) {
        match &mut self.node_data {
            NodeData::Leaf(x) => {
                if x.len() <= MAX_LEAF_ITEMS {
                    x.push((*point.xy(), data));
                } else {
                    self.subdivide();
                    self.insert(point, data);
                }
            }
            x => {
                if let Some(child) = x.child_nodes_mut().find(|c| c.contains_point(point)) {
                    child.insert(point, data);
                };
            }
        }
        // println!("{:?}", self.node_data);
    }
    pub fn contains_point<P: Point>(&self, point: P) -> bool {
        self.bounds.contains(point)
    }

    pub fn query(&self, bounds: Bounds) -> Vec<&([f32; 2], UserData)> {
        let mut result = Vec::new();
        if !self.bounds.overlaps(bounds) {
            return result;
        }

        match &self.node_data {
            NodeData::Branch {
                top_left,
                top_right,
                bottom_left,
                bottom_right,
            } => {
                result.extend(top_left.query(bounds));
                result.extend(top_right.query(bounds));
                result.extend(bottom_left.query(bounds));
                result.extend(bottom_right.query(bounds));
            }
            NodeData::Leaf(points) => {
                for point in points {
                    if bounds.contains(point.0) {
                        result.push(point);
                    }
                }
            }
        }
        result
    }

    pub fn query_distance<P: Point>(&self, point: P, distance: f32) -> Vec<&([f32; 2], UserData)> {
        let distance_squared = distance * distance;
        let point = point.xy();
        self.query(Bounds {
            x_min: point[0] - distance,
            x_max: point[0] + distance,
            y_min: point[1] - distance,
            y_max: point[1] + distance,
        })
        .into_iter()
        .filter(|(p, _)| {
            let a = p[0] - point[0];
            let b = p[1] - point[1];
            (a * a) + (b * b) < distance_squared
        })
        .collect()
    }

    fn subdivide(&mut self) {
        let bounds = self.bounds;
        let half_x = (bounds.x_max - bounds.x_min) / 2.0;
        let half_y = (bounds.y_max - bounds.y_min) / 2.0;
        let old = mem::replace(
            &mut self.node_data,
            NodeData::Branch {
                top_left: Box::new(QuadTree {
                    node_data: NodeData::empty_leaf(),
                    bounds: Bounds {
                        x_min: bounds.x_min,
                        x_max: bounds.x_min + half_x,
                        y_min: bounds.y_min,
                        y_max: bounds.y_min + half_y,
                    },
                }),
                top_right: Box::new(QuadTree {
                    node_data: NodeData::empty_leaf(),
                    bounds: Bounds {
                        x_min: bounds.x_min + half_x,
                        x_max: bounds.x_max,
                        y_min: bounds.y_min,
                        y_max: bounds.y_min + half_y,
                    },
                }),
                bottom_left: Box::new(QuadTree {
                    node_data: NodeData::empty_leaf(),
                    bounds: Bounds {
                        x_min: bounds.x_min,
                        x_max: bounds.x_min + half_x,
                        y_min: bounds.y_min + half_y,
                        y_max: bounds.y_max,
                    },
                }),
                bottom_right: Box::new(QuadTree {
                    node_data: NodeData::empty_leaf(),
                    bounds: Bounds {
                        x_min: bounds.x_min + half_x,
                        x_max: bounds.x_max,
                        y_min: bounds.y_min + half_y,
                        y_max: bounds.y_max,
                    },
                }),
            },
        );
        match old {
            NodeData::Leaf(x) => x.into_iter().for_each(|x| self.insert(x.0, x.1)),
            _ => panic!("subdivided branch node"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlaps() {
        let b1 = Bounds {
            x_min: 0.0,
            x_max: 100.0,
            y_min: 0.0,
            y_max: 100.0,
        };
        let b2 = Bounds {
            x_min: 50.0,
            x_max: 150.0,
            y_min: 50.0,
            y_max: 150.0,
        };
        let b3 = Bounds {
            x_min: 150.0,
            x_max: 250.0,
            y_min: 150.0,
            y_max: 250.0,
        };
        let b4 = Bounds {
            x_min: -100.0,
            x_max: -10.0,
            y_min: -100.0,
            y_max: -10.0,
        };
        assert!(b1.overlaps(b2));
        assert!(b2.overlaps(b3));
        assert!(!b1.overlaps(b3));
        assert!(!b1.overlaps(b4));
    }

    #[test]
    fn test_contains() {
        let b1 = Bounds {
            x_min: 0.0,
            x_max: 100.0,
            y_min: 0.0,
            y_max: 100.0,
        };

        assert!(b1.contains([50.0, 50.0]));
        assert!(!b1.contains([150.0, 150.0]));
        assert!(!b1.contains([-10.0, -10.0]));
    }
}

impl Point for Vec2 {
    fn xy(&self) -> &[f32; 2] {
        self.as_ref()
    }
}
