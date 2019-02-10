//! Node for sample and batch-based mixers.
//!

pub mod mixer_node;

use std::f64::consts::PI;

use crate::util::math;
use num::complex::Complex;
use num::Num;
use num_traits::NumCast;

pub struct Mixer {
    phase: f64,
    dphase: f64,
}

impl Mixer {
    pub fn new(phase: f64, mut dphase: f64) -> Mixer {
        while dphase >= 2.0 * PI {
            dphase -= 2.0 * PI;
        }
        while dphase < 0.0 {
            dphase += 2.0 * PI;
        }
        Mixer { phase, dphase }
    }

    pub fn mix<T>(&mut self, input: &Complex<T>) -> Complex<T>
    where
        T: NumCast + Clone + Num,
    {
        let inp: Complex<f64> = math::cast_complex(input).unwrap();
        let res = inp * Complex::exp(&Complex::new(0.0, self.phase));
        self.phase += self.dphase;
        if self.phase > 2.0 * PI {
            self.phase -= 2.0 * PI;
        }
        math::cast_complex(&res).unwrap()
    }
}
