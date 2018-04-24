//! Implementation of Vivaldi network coordinate system
//!
//! According to original paper:
//! Russ Cox, Frank Dabek, Frans Kaashoek, Jinyang Li, and Robert Morris. 2004. Practical,
//! distributed network coordinates. SIGCOMM Comput. Commun. Rev. 34, 1 (January 2004),
//! 113-118. DOI=http://dx.doi.org/10.1145/972374.972394
//!

use std::ops::{Add, Mul, Sub};

use rand::{Isaac64Rng, Rng};

use super::NodeCoordinates;

// todo set recommended in paper!
const NODE_ERROR_COEFF: f64 = 0.25; // C_c
const LOCAL_ERROR_WMA_COEFF: f64 = 0.5; // C_e

#[derive(Debug, Copy, Clone)]
struct HeightVector2D {
    x1: f64,
    x2: f64,
    height: f64,
}

impl<'a> From<&'a NodeCoordinates> for HeightVector2D {
    fn from(coord: &'a NodeCoordinates) -> Self {
        HeightVector2D {
            x1: coord.x1,
            x2: coord.x2,
            height: coord.height,
        }
    }
}

/// Vector + Vector
impl Add for HeightVector2D {
    type Output = Self;

    fn add(self, rhs: HeightVector2D) -> Self::Output {
        HeightVector2D {
            x1: self.x1 + rhs.x1,
            x2: self.x2 + rhs.x2,
            height: self.height + rhs.height,
        }
    }
}

/// Vector - Vector
impl Sub for HeightVector2D {
    type Output = Self;

    fn sub(self, rhs: HeightVector2D) -> Self::Output {
        HeightVector2D {
            x1: self.x1 - rhs.x1,
            x2: self.x2 - rhs.x2,
            height: self.height + rhs.height,
        }
    }
}

/// Scalar x Vector
impl Mul<f64> for HeightVector2D {
    type Output = HeightVector2D;

    fn mul(self, rhs: f64) -> Self::Output {
        HeightVector2D {
            x1: self.x1 * rhs,
            x2: self.x2 * rhs,
            height: self.height * rhs,
        }
    }
}

impl HeightVector2D {
    pub fn norm(self) -> f64 {
        (self.x1.powi(2) + self.x2.powi(2)).sqrt() + self.height
    }

    pub fn unit<R: Rng>(self, rng: &mut R) -> Self {
        let flat_vec_norm = HeightVector2D {
            x1: self.x1,
            x2: self.x2,
            height: 0.0,
        }.norm();

        if flat_vec_norm < 1e-9 {
            // generate random direction vector
            return HeightVector2D {
                x1: rng.next_f64(),
                x2: rng.next_f64(),
                height: 0.0,
            }.unit(rng);
        }

        HeightVector2D {
            x1: self.x1 / flat_vec_norm,
            x2: self.x2 / flat_vec_norm,
            height: 0.0,
        }
    }
}

/* Computation */

pub fn compute_location<R: Rng>(
    local_node: &NodeCoordinates,
    remote_node: &NodeCoordinates,
    rtt_sec: f64,
    rng: &mut R,
) -> NodeCoordinates {
    let sample_weight = local_node.pos_err / (local_node.pos_err + remote_node.pos_err);

    let computed_distance = node_distance(local_node, remote_node);

    let sample_err = (computed_distance - rtt_sec).abs() / rtt_sec;

    let new_pos_err = sample_err * LOCAL_ERROR_WMA_COEFF * sample_weight
        + local_node.pos_err * (1.0 - LOCAL_ERROR_WMA_COEFF * sample_weight);

    let timestep = NODE_ERROR_COEFF * sample_weight;

    let new_pos_vec = HeightVector2D::from(local_node)
        + (HeightVector2D::from(local_node) - HeightVector2D::from(remote_node)).unit(rng)
            * timestep * computed_distance;

    NodeCoordinates {
        x1: new_pos_vec.x1,
        x2: new_pos_vec.x2,
        height: new_pos_vec.height,
        pos_err: new_pos_err,
    }
}

/// Distance between two nodes in height-vector augmented Euclidean space
fn node_distance(n1: &NodeCoordinates, n2: &NodeCoordinates) -> f64 {
    (HeightVector2D::from(n1) - HeightVector2D::from(n2)).norm()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_vector_norm() {
        let vec = HeightVector2D {
            x1: 3.0,
            x2: 4.0,
            height: 0.5,
        };

        assert_eq!(vec.norm(), 5.5);
    }

    #[test]
    fn height_vector_unit() {
        let mut rng = Isaac64Rng::new_unseeded();
        let vec = HeightVector2D {
            x1: 3.0,
            x2: 4.0,
            height: 0.5,
        };

        let unit = vec.unit(&mut rng);

        assert_eq!(unit.norm(), 1.0);
        assert_eq!(unit.height, 0.0);
    }

    #[test]
    fn height_vector_unit_zero() {
        let mut rng = Isaac64Rng::new_unseeded();
        let zero_vec = HeightVector2D {
            x1: 0.0,
            x2: 0.0,
            height: 0.0,
        };

        let unit = zero_vec.unit(&mut rng);

        assert_eq!(zero_vec.norm(), 0.0);
        assert_eq!(unit.norm(), 1.0);
    }
}
