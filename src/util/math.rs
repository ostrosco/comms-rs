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
