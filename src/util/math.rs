use num::{Complex, Num};
use num_traits::AsPrimitive;

/// Casts a Complex<T> to a Complex<U>. All of the normal caveats with using
/// the `as` keyword apply here for the conversion.
pub fn cast_complex<T, U>(input: Complex<T>) -> Complex<U>
where
    T: AsPrimitive<U> + Clone + Num,
    U: Clone + Copy + Num + 'static,
{
    Complex::new(input.re.as_(), input.im.as_())
}

#[cfg(test)]
mod test {
    use num::Complex;
    use util::math;

    #[test]
    fn test_cast_complex() {
        let val = Complex::new(3.0, 4.0);
        let new_val: Complex<u8> = math::cast_complex(val);
        assert_eq!(new_val, Complex::new(3u8, 4u8));
        let new_new_val: Complex<f32> = math::cast_complex(new_val);
        assert_eq!(new_new_val, val);
    }
}
