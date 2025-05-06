use num::{Complex, Num, NumCast};
use std::f64::consts::PI;

use crate::util::MathError;

/// Casts a `Complex<T>` to a `Complex<U>`.
///
/// All of the normal caveats with using the `as` keyword apply here for the
/// conversion.
///
/// # Examples
///
/// ```
/// use comms_rs::util::math::cast_complex;
/// use num::Complex;
///
/// let a_num = Complex::new(12_i16, -4_i16);
/// let a_new_num: Complex<f64> = cast_complex(&a_num).unwrap();
/// ```
pub fn cast_complex<T, U>(input: &Complex<T>) -> Option<Complex<U>>
where
    T: Copy + Num + NumCast,
    U: Copy + Num + NumCast,
{
    let re = U::from(input.re)?;
    let im = U::from(input.im)?;
    Some(Complex::new(re, im))
}

/// Rectangle pulse shaping tap calculator.
///
/// Use this to create the taps for an FIR filter node and use that for the
/// pulse shaping node.
///
/// # Arguments
///
/// * `n_taps` - Number of desired output taps
///
/// # Examples
///
/// ```
/// use comms_rs::util::math::rect_taps;
/// use num::Complex;
///
/// let n_taps = 12_usize;
/// let taps: Vec<Complex<f64>> = rect_taps(n_taps).unwrap();
/// ```
pub fn rect_taps<T>(n_taps: usize) -> Option<Vec<Complex<T>>>
where
    T: Copy + Num + NumCast,
{
    let re = T::from(1)?;
    let im = T::from(0)?;
    Some(vec![Complex::new(re, im); n_taps as usize])
}

/// Gaussian filter impulse response.
///
/// Use this to create the taps for an FIR filter node and use that for a pulse
/// shaping node.
///
/// # Arguments
///
/// * `n_taps` - Number of desired output taps
/// * `sam_per_sym` - Samples per symbol
/// * `alpha` - Shaping parameter of the function
///
/// # Example
///
/// ```
/// use comms_rs::util::math::gaussian_taps;
/// use num::Complex;
///
/// let n_taps = 28_u32;
/// let sams_per_sym = 4.0_f64;
/// let alpha = 0.25_f64;
/// let taps: Vec<Complex<f64>> = gaussian_taps(n_taps, sams_per_sym, alpha).unwrap();
/// ```
pub fn gaussian_taps<T>(
    n_taps: u32,
    sam_per_sym: f64,
    alpha: f64,
) -> Option<Vec<Complex<T>>>
where
    T: Copy + Num + NumCast,
{
    let tsym: f64 = 1.0;
    let fs: f64 = sam_per_sym / tsym;

    let f =
        |t: f64| -> f64 { (alpha / PI).sqrt() * (-alpha * t.powi(2)).exp() };

    let mut taps = Vec::new();
    for i in 0..n_taps {
        let t = (i as f64 - (n_taps - 1) as f64 / 2.0) / fs;
        let value = T::from(f(t))?;
        let im = T::from(0)?;
        taps.push(Complex::new(value, im));
    }

    Some(taps)
}

/// Normalized sinc function implementation.
///
/// `sinc(0) = 1`
///
/// `sinc(x) = sin(pi * x) / (pi * x), x != 0`
///
/// # Examples
///
/// ```
/// use comms_rs::util::math::sinc;
///
/// assert!((sinc(0.0) - 1.0).abs() < std::f64::EPSILON);
/// assert!(sinc(1.0).abs() < std::f64::EPSILON);
/// assert!(sinc(2.0).abs() < std::f64::EPSILON);
/// assert!(sinc(3.0).abs() < std::f64::EPSILON);
/// ```
pub fn sinc(x: f64) -> f64 {
    if x != 0.0 {
        (PI * x).sin() / (PI * x)
    } else {
        1.0
    }
}

/// Raise Cosine (RC) filter tap calculator.
///
/// Use this to create the taps for a FIR filter node and use that as your
/// pulse shaping.
///
/// # Arguments
///
/// * `n_taps` - Number of desired output taps
/// * `sam_per_sym` - Samples per symbol
/// * `beta` - Shaping parameter of the RC function.  Must be on the interval
///            [0, 1].
///
/// # Examples
///
/// ```
/// use comms_rs::util::math::rc_taps;
/// use num::Complex;
///
/// let n_taps = 28u32;
/// let sams_per_sym = 4.0_f64;
/// let beta = 0.25_f64;
/// let taps: Vec<Complex<f64>> = rc_taps(n_taps, sams_per_sym, beta).unwrap();
/// ```
pub fn rc_taps<T>(
    n_taps: u32,
    sam_per_sym: f64,
    beta: f64,
) -> Result<Vec<Complex<T>>, MathError>
where
    T: Copy + Num + NumCast,
{
    if !(0.0..=1.0).contains(&beta) {
        return Err(MathError::InvalidRolloffError);
    };

    let tsym: f64 = 1.0;
    let fs: f64 = sam_per_sym / tsym;

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

        let im = T::from(0).ok_or(MathError::ConvertError)?;
        if (t - zero_denom).abs() < std::f64::EPSILON
            || (t + zero_denom).abs() < std::f64::EPSILON
        {
            let re = T::from(fint()).ok_or(MathError::ConvertError)?;
            taps.push(Complex::new(re, im));
        } else {
            let re = T::from(f(t)).ok_or(MathError::ConvertError)?;
            taps.push(Complex::new(re, im));
        }
    }

    Ok(taps)
}

/// Root Raised Cosine (RRC) filter tap calculator.
///
/// Use this to create the taps for an FIR filter node and use that as your
/// pulse shaping.
///
/// # Arguments
///
/// * `n_taps` - Number of desired output taps
/// * `sam_per_sym` - Samples per symbol
/// * `beta` - Shaping parameter of the RRC function.  Must be on the interval
///            [0.0, 1.0].
///
/// # Examples
///
/// ```
/// use comms_rs::util::math::rrc_taps;
/// use num::Complex;
///
/// let n_taps = 28u32;
/// let sams_per_sym = 4.0_f64;
/// let beta = 0.25_f64;
/// let taps: Vec<Complex<f64>> = rrc_taps(n_taps, sams_per_sym, beta).unwrap();
/// ```
pub fn rrc_taps<T>(
    n_taps: u32,
    sam_per_sym: f64,
    beta: f64,
) -> Result<Vec<Complex<T>>, MathError>
where
    T: Copy + Num + NumCast,
{
    if !(0.0..=1.0).contains(&beta) {
        return Err(MathError::InvalidRolloffError);
    };

    let tsym: f64 = 1.0;
    let fs: f64 = sam_per_sym / tsym;

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

    let zero_denom: f64 = if beta != 0.0 {
        tsym / (4.0 * beta)
    } else {
        0.0
    };

    let mut taps = Vec::new();
    for i in 0..n_taps {
        let t: f64 = (i as f64 - (n_taps - 1) as f64 / 2.0) / fs;

        let im = T::from(0).ok_or(MathError::ConvertError)?;
        if t.abs() < std::f64::EPSILON {
            let re = T::from(fzero()).ok_or(MathError::ConvertError)?;
            taps.push(Complex::new(re, im));
        } else if (t - zero_denom).abs() < std::f64::EPSILON
            || (t + zero_denom).abs() < std::f64::EPSILON
        {
            let re = T::from(fint()).ok_or(MathError::ConvertError)?;
            taps.push(Complex::new(re, im));
        } else {
            let re = T::from(f(t)).ok_or(MathError::ConvertError)?;
            taps.push(Complex::new(re, im));
        }
    }

    Ok(taps)
}

/// Implementation of the Mengali qfilter tap calculator.
///
/// This is specifically to support Mengali's feedforward non data aided
/// maximum likelihood estimator described in chp. 8.4 of his book as q(t).
///
/// # Arguments
///
/// * `n_taps` - Number of desired output taps.  Only takes odd numbers.  Even
///              numbers will be incremented by one and that shall be used
///              intead.
/// * `alpha` - Shaping parameter of the function. Must be on the interval
///             [0.0, 1.0].
/// * `sam_per_sym` - Samples per symbol
///
/// # Examples
///
/// ```
/// use comms_rs::util::math::qfilt_taps;
/// use num::Complex;
///
/// let n_taps = 21;
/// let alpha = 0.25;
/// let sams_per_sym = 2;
/// let taps: Vec<f64> = qfilt_taps(n_taps, alpha, sams_per_sym).unwrap();
/// ```
pub fn qfilt_taps(
    n_taps: u32,
    alpha: f64,
    sam_per_sym: u32,
) -> Result<Vec<f64>, MathError> {
    if !(0.0..=1.0).contains(&alpha) {
        return Err(MathError::InvalidRolloffError);
    };

    // We want an odd number of taps
    let mut real_n_taps = n_taps;
    if n_taps % 2 == 0 {
        real_n_taps += 1;
    }

    let d = ((real_n_taps as f64) / 2.0).floor() as i32;
    let ttarr: Vec<f64> = (0..real_n_taps)
        .map(|x| (x as i32 - d) as f64 / (sam_per_sym as f64))
        .collect();

    let mut output = vec![];
    for tt in ttarr {
        let two_alpha_tt = 2.0 * alpha * tt;
        #[allow(clippy::float_cmp)]
        let use_lhospitals = two_alpha_tt.abs() == 1.0;
        if use_lhospitals {
            output.push((PI * alpha * tt).sin() / (8.0 * tt));
        } else {
            let numerator = alpha * (PI * alpha * tt).cos();
            let denominator = PI * (1.0 - (two_alpha_tt * two_alpha_tt));
            output.push(numerator / denominator);
        }
    }

    Ok(output)
}

#[allow(clippy::excessive_precision)]
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

        let test = math::rrc_taps(33, 3.18, 0.234).unwrap();

        for i in 0..truth.len() {
            assert!((truth[i] - test[i]).norm() < std::f32::EPSILON);
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

        let test: Vec<_> = math::rc_taps(33, 3.18, 0.234).unwrap();
        for i in 0..truth.len() {
            assert!((truth[i] - test[i]).norm() < std::f32::EPSILON);
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

        let test: Vec<_> = math::gaussian_taps(33, 3.18, 0.234).unwrap();
        for i in 0..truth.len() {
            assert!((truth[i] - test[i]).norm() < std::f32::EPSILON);
        }
    }

    #[test]
    fn test_qfilt_taps() {
        let truth = vec![
            Complex::new(0.010718051382822693, 0.0),
            Complex::new(0.018097230082535474, 0.0),
            Complex::new(0.026525823848649224, 0.0),
            Complex::new(0.03564605925347896, 0.0),
            Complex::new(0.045015815807855304, 0.0),
            Complex::new(0.05413863102246848, 0.0),
            Complex::new(0.0625, 0.0),
            Complex::new(0.06960681131460235, 0.0),
            Complex::new(0.07502635967975885, 0.0),
            Complex::new(0.07842133035765372, 0.0),
            Complex::new(0.07957747154594767, 0.0),
            Complex::new(0.07842133035765372, 0.0),
            Complex::new(0.07502635967975885, 0.0),
            Complex::new(0.06960681131460235, 0.0),
            Complex::new(0.0625, 0.0),
            Complex::new(0.05413863102246848, 0.0),
            Complex::new(0.045015815807855304, 0.0),
            Complex::new(0.03564605925347896, 0.0),
            Complex::new(0.026525823848649224, 0.0),
            Complex::new(0.018097230082535474, 0.0),
        ];

        let test: Vec<_> = math::qfilt_taps(21, 0.25, 2).unwrap();
        for i in 0..truth.len() {
            assert!((truth[i] - test[i]).norm() < std::f64::EPSILON);
        }
    }
}
