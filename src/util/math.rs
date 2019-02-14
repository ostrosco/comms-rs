use num::{Complex, Num};
use num_traits::NumCast;
use std::f64::consts::PI;

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

/// Rectangle pulse shaping tap calculator.  Use this to create the taps for an
/// FIR filter node and use that for the pulse shaping node.
///
/// Arguments:
/// n_taps: Number of desired output taps
pub fn rect_taps(n_taps: u32) -> Vec<Complex<f64>> {
    vec![Complex::new(1.0, 0.0); n_taps as usize]
}

/// Gaussian filter impulse response.  Use this to create the taps for an FIR
/// filter node and use that for a pulse shaping node.
///
/// Arguments:
/// n_taps: Number of desired output taps
/// sam_per_sym: Samples per symbol
/// beta: Shaping parameter of the RC function
pub fn gaussian_taps(
    n_taps: u32,
    sam_per_sym: f64,
    alpha: f64,
) -> Vec<Complex<f64>> {
    let tsym = 1.0_f64;
    let fs = sam_per_sym / tsym;

    let f =
        |t: f64| -> f64 { (alpha / PI).sqrt() * (-alpha * t.powi(2)).exp() };

    let mut taps = Vec::new();
    for i in 0..n_taps {
        let t = (i as f64 - (n_taps - 1) as f64 / 2.0) / fs;
        taps.push(Complex::new(f(t), 0.0));
    }

    taps
}

/// Normalized sinc function implementation
/// sinc(0) = 1
/// sinc(x) = sin(pi * x) / (pi * x), x != 0
pub fn sinc(x: f64) -> f64 {
    if x != 0.0 {
        (PI * x).sin() / (PI * x)
    } else {
        1.0
    }
}

/// Raise Cosine (RC) filter tap calculator. Use this to create the taps for a
/// FIR filter node and use that as your pulse shaping.
///
/// Arguments:
/// n_taps: Number of desired output taps
/// sam_per_sym: Samples per symbol
/// beta: Shaping parameter of the RC function
pub fn rc_taps(n_taps: u32, sam_per_sym: f64, beta: f64) -> Vec<Complex<f64>> {
    let tsym = 1.0_f64;
    let fs = sam_per_sym / tsym;

    let fint = || -> f64 { (PI / (4.0 * tsym)) * sinc(1.0 / (2.0 * beta)) };

    let f = |t: f64| -> f64 {
        (1.0 / tsym) * sinc(t / tsym) * ((PI * beta * t) / tsym).cos()
            / (1.0 - ((2.0 * beta * t) / tsym).powi(2))
    };

    let zero_denom = if beta != 0.0 {
        tsym / (2.0 * beta)
    } else {
        0.0
    };

    let mut taps = Vec::new();
    for i in 0..n_taps {
        let t = (i as f64 - (n_taps - 1) as f64 / 2.0) / fs;

        if (t - zero_denom).abs() < std::f64::EPSILON
            || (t + zero_denom).abs() < std::f64::EPSILON
        {
            taps.push(Complex::new(fint(), 0.0));
        } else {
            taps.push(Complex::new(f(t), 0.0));
        }
    }

    taps
}

/// Root Raised Cosine (RRC) filter tap calculator.  Use this to create the
/// taps for an FIR filter node and use that as your pulse shaping.
///
/// Arguments:
/// n_taps: Number of desired output taps
/// sam_per_sym: Samples per symbol
/// beta: Shaping parameter of the RRC function
pub fn rrc_taps(n_taps: u32, sam_per_sym: f64, beta: f64) -> Vec<Complex<f64>> {
    let tsym = 1.0_f64;
    let fs = sam_per_sym / tsym;

    let fzero = || -> f64 { (1.0 / tsym) * (1.0 + beta * (4.0 / PI - 1.0)) };

    let fint = || -> f64 {
        (beta / (tsym * 2.0_f64.sqrt()))
            * ((1.0 + 2.0 / PI) * (PI / (4.0 * beta)).sin()
                + (1.0 - (2.0 / PI)) * (PI / (4.0 * beta)).cos())
    };

    let f = |t: f64| -> f64 {
        (1.0 / tsym)
            * ((PI * (t / tsym) * (1.0 - beta)).sin()
                + 4.0
                    * beta
                    * (t / tsym)
                    * (PI * (t / tsym) * (1.0 + beta)).cos())
            / (PI * (t / tsym) * (1.0 - (4.0 * beta * (t / tsym)).powi(2)))
    };

    let zero_denom = if beta != 0.0 {
        tsym / (4.0 * beta)
    } else {
        0.0
    };

    let mut taps = Vec::new();
    for i in 0..n_taps {
        let t = (i as f64 - (n_taps - 1) as f64 / 2.0) / fs;

        if t.abs() < std::f64::EPSILON {
            taps.push(Complex::new(fzero(), 0.0));
        } else if (t - zero_denom).abs() < std::f64::EPSILON
            || (t + zero_denom).abs() < std::f64::EPSILON
        {
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

    #[test]
    fn test_rrc_taps() {
        let truth = vec![
            Complex::new(-0.00982617, 0.0),
            Complex::new(-0.01020256, 0.0),
            Complex::new(0.00807753, 0.0),
            Complex::new(0.02391673, 0.0),
            Complex::new(0.00978662, 0.0),
            Complex::new(-0.02953471, 0.0),
            Complex::new(-0.0488395, 0.0),
            Complex::new(-0.00823527, 0.0),
            Complex::new(0.06838797, 0.0),
            Complex::new(0.09486125, 0.0),
            Complex::new(0.00622454, 0.0),
            Complex::new(-0.14706016, 0.0),
            Complex::new(-0.20088982, 0.0),
            Complex::new(-0.00457254, 0.0),
            Complex::new(0.42552248, 0.0),
            Complex::new(0.87324694, 0.0),
            Complex::new(1.06393805, 0.0),
            Complex::new(0.87324694, 0.0),
            Complex::new(0.42552248, 0.0),
            Complex::new(-0.00457254, 0.0),
            Complex::new(-0.20088982, 0.0),
            Complex::new(-0.14706016, 0.0),
            Complex::new(0.00622454, 0.0),
            Complex::new(0.09486125, 0.0),
            Complex::new(0.06838797, 0.0),
            Complex::new(-0.00823527, 0.0),
            Complex::new(-0.0488395, 0.0),
            Complex::new(-0.02953471, 0.0),
            Complex::new(0.00978662, 0.0),
            Complex::new(0.02391673, 0.0),
            Complex::new(0.00807753, 0.0),
            Complex::new(-0.01020256, 0.0),
            Complex::new(-0.00982617, 0.0),
        ];

        let test = math::rrc_taps(33, 3.18, 0.234);

        let epsilon = 0.00000001;
        for i in 0..truth.len() {
            assert!((truth[i] - test[i]).norm() < epsilon);
        }
    }

    #[test]
    fn test_rc_taps() {
        let truth = vec![
            Complex::new(-0.0011653229685676335, 0.0),
            Complex::new(0.012816317493783883, 0.0),
            Complex::new(0.021147755355340796, 0.0),
            Complex::new(0.00791903759636216, 0.0),
            Complex::new(-0.024253219358036038, 0.0),
            Complex::new(-0.0465161104657352, 0.0),
            Complex::new(-0.025723996627094965, 0.0),
            Complex::new(0.036996624996837396, 0.0),
            Complex::new(0.08999421769005823, 0.0),
            Complex::new(0.06609535709951565, 0.0),
            Complex::new(-0.048727623832534546, 0.0),
            Complex::new(-0.17340916580147755, 0.0),
            Complex::new(-0.16888992011002318, 0.0),
            Complex::new(0.05701023237025582, 0.0),
            Complex::new(0.4558112530148015, 0.0),
            Complex::new(0.8408212451367716, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(0.8408212451367716, 0.0),
            Complex::new(0.4558112530148015, 0.0),
            Complex::new(0.05701023237025582, 0.0),
            Complex::new(-0.16888992011002318, 0.0),
            Complex::new(-0.17340916580147755, 0.0),
            Complex::new(-0.048727623832534546, 0.0),
            Complex::new(0.06609535709951565, 0.0),
            Complex::new(0.08999421769005823, 0.0),
            Complex::new(0.036996624996837396, 0.0),
            Complex::new(-0.025723996627094965, 0.0),
            Complex::new(-0.0465161104657352, 0.0),
            Complex::new(-0.024253219358036038, 0.0),
            Complex::new(0.00791903759636216, 0.0),
            Complex::new(0.021147755355340796, 0.0),
            Complex::new(0.012816317493783883, 0.0),
            Complex::new(-0.0011653229685676335, 0.0),
        ];

        let test: Vec<_> = math::rc_taps(33, 3.18, 0.234);
        let epsilon = 0.00000001;
        for i in 0..truth.len() {
            assert!((truth[i] - test[i]).norm() < epsilon);
        }
    }

    #[test]
    fn test_gaussian_taps() {
        let truth = vec![
            Complex::new(0.0007300494185482611, 0.0),
            Complex::new(0.0014958492117118187, 0.0),
            Complex::new(0.0029263367824777266, 0.0),
            Complex::new(0.005465900570629832, 0.0),
            Complex::new(0.0097476534361888, 0.0),
            Complex::new(0.016597373400549398, 0.0),
            Complex::new(0.02698233817269414, 0.0),
            Complex::new(0.041881355492128326, 0.0),
            Complex::new(0.06206729366026605, 0.0),
            Complex::new(0.08782250506026018, 0.0),
            Complex::new(0.11864508840813756, 0.0),
            Complex::new(0.15303636428781775, 0.0),
            Complex::new(0.1884692257990131, 0.0),
            Complex::new(0.22160889352023885, 0.0),
            Complex::new(0.248791108947204, 0.0),
            Complex::new(0.26667570890130865, 0.0),
            Complex::new(0.27291851048803384, 0.0),
            Complex::new(0.26667570890130865, 0.0),
            Complex::new(0.248791108947204, 0.0),
            Complex::new(0.22160889352023885, 0.0),
            Complex::new(0.1884692257990131, 0.0),
            Complex::new(0.15303636428781775, 0.0),
            Complex::new(0.11864508840813756, 0.0),
            Complex::new(0.08782250506026018, 0.0),
            Complex::new(0.06206729366026605, 0.0),
            Complex::new(0.041881355492128326, 0.0),
            Complex::new(0.02698233817269414, 0.0),
            Complex::new(0.016597373400549398, 0.0),
            Complex::new(0.0097476534361888, 0.0),
            Complex::new(0.005465900570629832, 0.0),
            Complex::new(0.0029263367824777266, 0.0),
            Complex::new(0.0014958492117118187, 0.0),
            Complex::new(0.0007300494185482611, 0.0),
        ];

        let test: Vec<_> = math::gaussian_taps(33, 3.18, 0.234);
        let epsilon = 0.00000001;
        for i in 0..truth.len() {
            assert!((truth[i] - test[i]).norm() < epsilon);
        }
    }
}
