use crate::num::Zero;
use num::complex::Complex;
use std::collections::VecDeque;

/// Constructs a cubic interpolation filter with the Farrow structure as
/// detailed in "Interpolation in Digital Modems -- Part II: Implementation and Performance"
/// by Erup, Gardner, and Harris, 1993
///
/// # Arguments
///
/// * `sample` - Input complex sample.
///
/// # Examples
///
/// ```
/// use comms_rs::demodulation::farrow_filter::*;
/// use num::Complex;
///
/// let samples: Vec<_> = (0..100).map(|x| Complex::new(0.0, x as f64).exp()).collect();
///
/// let mut filter = FarrowFilter::new();
///
/// let mut output = Vec::new();
/// for s in samples {
///     output.push(filter.push(s));
/// }
/// ```
pub struct FarrowFilter {
    buffer: VecDeque<Complex<f64>>,
    coeffs: [[Complex<f64>; 4]; 4],
    pub mu: f64,
}

impl Default for FarrowFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl FarrowFilter {
    pub fn new() -> FarrowFilter {
        let buffer = VecDeque::from([
            Complex::zero(),
            Complex::zero(),
            Complex::zero(),
            Complex::zero(),
        ]);

        FarrowFilter {
            buffer,
            coeffs: [
                [
                    Complex::zero(),
                    Complex::zero(),
                    Complex::new(1.0, 0.0),
                    Complex::zero(),
                ],
                [
                    Complex::new(-1.0 / 6.0, 0.0),
                    Complex::new(1.0, 0.0),
                    Complex::new(-0.5, 0.0),
                    Complex::new(-1.0 / 3.0, 0.0),
                ],
                [
                    Complex::zero(),
                    Complex::new(0.5, 0.0),
                    Complex::new(-1.0, 0.0),
                    Complex::new(0.5, 0.0),
                ],
                [
                    Complex::new(1.0 / 6.0, 0.0),
                    Complex::new(-0.5, 0.0),
                    Complex::new(0.5, 0.0),
                    Complex::new(-1.0 / 6.0, 0.0),
                ],
            ],
            mu: 0.0,
        }
    }

    pub fn push(&mut self, sample: Complex<f64>) -> Complex<f64> {
        // Push newest sample into ring buffer
        self.buffer.pop_back();
        self.buffer.push_front(sample);

        // Calculate v0, v1, v2, v3
        let v0: Complex<f64> = self
            .buffer
            .iter()
            .zip(self.coeffs[0].iter())
            .map(|(b, c)| b * c)
            .sum();
        let v1: Complex<f64> = self
            .buffer
            .iter()
            .zip(self.coeffs[1].iter())
            .map(|(b, c)| b * c)
            .sum();
        let v2: Complex<f64> = self
            .buffer
            .iter()
            .zip(self.coeffs[2].iter())
            .map(|(b, c)| b * c)
            .sum();
        let v3: Complex<f64> = self
            .buffer
            .iter()
            .zip(self.coeffs[3].iter())
            .map(|(b, c)| b * c)
            .sum();

        // Calculate y = v0 + mu * (v1 + mu * (v2 + mu * v3))
        v0 + self.mu * (v1 + self.mu * (v2 + self.mu * v3))
    }
}

#[cfg(test)]
mod test {
    use crate::demodulation::farrow_filter::*;

    #[test]
    fn test_farrow_filter() {
        let samples = vec![
            Complex::new(0.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
        ];

        let mut filter = FarrowFilter::default();
        filter.mu = 0.5;

        let mut output = vec![];
        for s in samples {
            output.push(filter.push(s));
        }

        let epsilon = 1e-6;
        assert!((output[0] - 0.0).norm() < epsilon);
        assert!((output[1] + 0.0625).norm() < epsilon);
        assert!((output[2] - 0.5).norm() < epsilon);
        assert!((output[3] - 1.0625).norm() < epsilon);
        assert!((output[4] - 1.0).norm() < epsilon);
        assert!((output[5] - 1.0625).norm() < epsilon);
        assert!((output[6] - 0.5).norm() < epsilon);
        assert!((output[7] + 0.0625).norm() < epsilon);
        assert!((output[8] - 0.0).norm() < epsilon);
    }
}
