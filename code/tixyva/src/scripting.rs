//! Has a module with extra fun math functions and stuff.

use rhai::plugin::*;

#[export_module]
pub mod tixyva_utils {
    use core::f64;

    pub fn atan2(y: f64, x: f64) -> f64 {
        y.atan2(x)
    }

    pub fn min(x: f64, y: f64) -> f64 {
        x.min(y)
    }

    pub fn max(x: f64, y: f64) -> f64 {
        x.max(y)
    }

    pub fn clamp(x: f64, min: f64, max: f64) -> f64 {
        if x <= min {
            min
        } else if x >= max {
            max
        } else {
            x
        }
    }

    pub fn rand() -> f64 {
        fastrand::f64()
    }
}
