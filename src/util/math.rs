use num::{Complex, Num};
use num_traits::NumCast;

/// Casts a Complex<T> to a Complex<U>. All of the normal caveats with using
/// the `as` keyword apply here for the conversion.
pub fn cast_complex<T, U>(input: &Complex<T>) -> Option<Complex<U>>
where
    T: Clone + Num + NumCast,
    U: Clone + Num + NumCast,
{
    let re = U::from(input.re.clone())?;
    let im = U::from(input.im.clone())?;
    Some(Complex::new(re, im))
}

/// Root Raised Cosine (RRC) filter tap calculator.
/// n_taps: Number of desired output taps
/// sam_per_sym: Samples per symbol
/// beta: Shaping parameter of the RRC function
pub fn rrc_taps(n_taps: u32, sam_per_sym: f64, beta: f64) -> Vec<Complex<f64>> {
    let tsym = 1.0_f64;
    let fs = sam_per_sym / tsym;

    let fzero = || -> f64 {
        return (1.0 / tsym) * (1.0 + beta * (4.0 / std::f64::consts::PI - 1.0));
    };

    let fint = || -> f64 {
        return (beta / (tsym * 2.0_f64.sqrt())) * ((1.0 + 2.0 / std::f64::consts::PI) * (std::f64::consts::PI / (4.0 * beta)).sin() +
            (1.0 - (2.0 / std::f64::consts::PI)) * (std::f64::consts::PI / (4.0 * beta)).cos());
    };

    let f = |t: f64| -> f64 {
        return (1.0 / tsym) * ((std::f64::consts::PI * (t / tsym) * (1.0 - beta)).sin() +
            4.0 * beta * (t / tsym) * (std::f64::consts::PI * (t / tsym) * (1.0 + beta)).cos()) /
            (std::f64::consts::PI * (t / tsym) * (1.0 - (4.0 * beta * (t / tsym)).powi(2)));
    };

    let mut zero_denom = 0.0;
    if beta != 0.0 {
        zero_denom = tsym / (4.0 * beta);
    }

    let mut taps = Vec::new();
    for i in 0..n_taps {
        let t = (i as f64 - (n_taps - 1) as f64 / 2.0) / fs;

        if t.abs() < std::f64::EPSILON {
            taps.push(Complex::new(fzero(), 0.0));
        } else if (t - zero_denom).abs() < std::f64::EPSILON || (t + zero_denom).abs() < std::f64::EPSILON {
            taps.push(Complex::new(fint(), 0.0));
        } else {
            taps.push(Complex::new(f(t), 0.0));
        }
    }

    taps
}

#[cfg(test)]
mod test {
    use crate::util::math;
    use num::Complex;

    #[test]
    fn test_cast_complex() {
        let val = Complex::new(3.0, 4.0);
        let new_val: Complex<u8> = math::cast_complex(&val).unwrap();
        assert_eq!(new_val, Complex::new(3u8, 4u8));
        let new_new_val: Complex<f32> = math::cast_complex(&new_val).unwrap();
        assert_eq!(new_new_val, val);
    }
}
