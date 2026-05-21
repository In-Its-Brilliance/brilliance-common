use crate::{
    chunks::{block_position::BlockPosition, position::Vector3},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayBlockStep {
    pub pos: BlockPosition,
    pub distance: f32,
}

pub struct RayBlockIter {
    max_dist: f32,
    current: BlockPosition,
    traveled: f32,
    step_x: i64,
    step_y: i64,
    step_z: i64,
    t_max_x: f32,
    t_max_y: f32,
    t_max_z: f32,
    t_delta_x: f32,
    t_delta_y: f32,
    t_delta_z: f32,
    finished: bool,
}

impl RayBlockIter {
    pub fn new(origin: Vector3, dir: Vector3, max_dist: f32) -> Self {
        if max_dist <= 0.0 {
            return Self {
                max_dist,
                current: BlockPosition::from_position(&origin),
                traveled: 0.0,
                step_x: 0,
                step_y: 0,
                step_z: 0,
                t_max_x: f32::INFINITY,
                t_max_y: f32::INFINITY,
                t_max_z: f32::INFINITY,
                t_delta_x: f32::INFINITY,
                t_delta_y: f32::INFINITY,
                t_delta_z: f32::INFINITY,
                finished: true,
            };
        }

        let dir_len = (dir.x * dir.x + dir.y * dir.y + dir.z * dir.z).sqrt();
        if dir_len == 0.0 {
            panic!("for_each_block_on_ray: dir must be non-zero");
        }

        let dir = Vector3::new(dir.x / dir_len, dir.y / dir_len, dir.z / dir_len);
        let current = BlockPosition::from_position(&origin);

        let step_x = if dir.x > 0.0 {
            1_i64
        } else if dir.x < 0.0 {
            -1_i64
        } else {
            0_i64
        };
        let step_y = if dir.y > 0.0 {
            1_i64
        } else if dir.y < 0.0 {
            -1_i64
        } else {
            0_i64
        };
        let step_z = if dir.z > 0.0 {
            1_i64
        } else if dir.z < 0.0 {
            -1_i64
        } else {
            0_i64
        };

        let t_max_x = if step_x == 0 {
            f32::INFINITY
        } else {
            let current_x = current.get_position().x;
            let next_boundary = if step_x > 0 { current_x + 1.0 } else { current_x };
            (next_boundary - origin.x) / dir.x
        };
        let t_max_y = if step_y == 0 {
            f32::INFINITY
        } else {
            let current_y = current.get_position().y;
            let next_boundary = if step_y > 0 { current_y + 1.0 } else { current_y };
            (next_boundary - origin.y) / dir.y
        };
        let t_max_z = if step_z == 0 {
            f32::INFINITY
        } else {
            let current_z = current.get_position().z;
            let next_boundary = if step_z > 0 { current_z + 1.0 } else { current_z };
            (next_boundary - origin.z) / dir.z
        };

        let t_delta_x = if step_x == 0 { f32::INFINITY } else { 1.0 / dir.x.abs() };
        let t_delta_y = if step_y == 0 { f32::INFINITY } else { 1.0 / dir.y.abs() };
        let t_delta_z = if step_z == 0 { f32::INFINITY } else { 1.0 / dir.z.abs() };

        Self {
            max_dist,
            current,
            traveled: 0.0,
            step_x,
            step_y,
            step_z,
            t_max_x,
            t_max_y,
            t_max_z,
            t_delta_x,
            t_delta_y,
            t_delta_z,
            finished: false,
        }
    }
}

impl Iterator for RayBlockIter {
    type Item = RayBlockStep;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.traveled > self.max_dist {
            return None;
        }

        let result = RayBlockStep {
            pos: self.current,
            distance: self.traveled,
        };

        if self.t_max_x < self.t_max_y {
            if self.t_max_x < self.t_max_z {
                self.current = BlockPosition::new(
                    self.current.get_position().x as i64 + self.step_x,
                    self.current.get_position().y as i64,
                    self.current.get_position().z as i64,
                );
                self.traveled = self.t_max_x;
                self.t_max_x += self.t_delta_x;
            } else {
                self.current = BlockPosition::new(
                    self.current.get_position().x as i64,
                    self.current.get_position().y as i64,
                    self.current.get_position().z as i64 + self.step_z,
                );
                self.traveled = self.t_max_z;
                self.t_max_z += self.t_delta_z;
            }
        } else if self.t_max_y < self.t_max_z {
            self.current = BlockPosition::new(
                self.current.get_position().x as i64,
                self.current.get_position().y as i64 + self.step_y,
                self.current.get_position().z as i64,
            );
            self.traveled = self.t_max_y;
            self.t_max_y += self.t_delta_y;
        } else {
            self.current = BlockPosition::new(
                self.current.get_position().x as i64,
                self.current.get_position().y as i64,
                self.current.get_position().z as i64 + self.step_z,
            );
            self.traveled = self.t_max_z;
            self.t_max_z += self.t_delta_z;
        }

        if self.traveled > self.max_dist {
            self.finished = true;
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{RayBlockIter, RayBlockStep};
    use crate::chunks::{block_position::BlockPosition, position::Vector3};
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn collect(origin: Vector3, dir: Vector3, max_dist: f32, limit: usize) -> Vec<BlockPosition> {
        RayBlockIter::new(origin, dir, max_dist)
            .take(limit)
            .map(|step| step.pos)
            .collect()
    }

    #[test]
    fn walks_positive_x() {
        let got = collect(Vector3::new(0.2, 0.2, 0.2), Vector3::new(1.0, 0.0, 0.0), 10.0, 4);
        assert_eq!(
            got,
            vec![
                BlockPosition::new(0, 0, 0),
                BlockPosition::new(1, 0, 0),
                BlockPosition::new(2, 0, 0),
                BlockPosition::new(3, 0, 0),
            ]
        );
    }

    #[test]
    fn walks_negative_x() {
        let got = collect(Vector3::new(2.8, 0.2, 0.2), Vector3::new(-1.0, 0.0, 0.0), 10.0, 4);
        assert_eq!(
            got,
            vec![
                BlockPosition::new(2, 0, 0),
                BlockPosition::new(1, 0, 0),
                BlockPosition::new(0, 0, 0),
                BlockPosition::new(-1, 0, 0),
            ]
        );
    }

    #[test]
    fn stops_on_max_dist_zero() {
        let got = collect(Vector3::new(0.2, 0.2, 0.2), Vector3::new(1.0, 0.0, 0.0), 0.0, 4);
        assert!(got.is_empty());
    }

    #[test]
    fn panics_on_zero_direction() {
        let result = catch_unwind(AssertUnwindSafe(|| {
            RayBlockIter::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0), 10.0);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn ray_step_has_distance() {
        let mut iter = RayBlockIter::new(Vector3::new(0.2, 0.2, 0.2), Vector3::new(1.0, 0.0, 0.0), 10.0);
        let first = iter.next().unwrap();
        let second = iter.next().unwrap();

        assert_eq!(
            first,
            RayBlockStep {
                pos: BlockPosition::new(0, 0, 0),
                distance: 0.0,
            }
        );
        assert_eq!(
            second,
            RayBlockStep {
                pos: BlockPosition::new(1, 0, 0),
                distance: 0.8,
            }
        );
    }
}
