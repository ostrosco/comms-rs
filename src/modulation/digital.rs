//! Provide tools to do digital modulation

use num::complex::Complex;

/// Modulates a bit to a complex int16 impulse via BPSK
pub fn bpsk_bit_mod(bit: u8) -> Option<Complex<i16>> {
    if bit == 0 {
        Some(Complex::new(1, 0))
    } else if bit == 1 {
        Some(Complex::new(-1, 0))
    } else {
        None
    }
}

/// Modulates a byte via BPSK into 8 int16 samples
pub fn bpsk_byte_mod(byte: u8) -> Vec<Complex<i16>> {
    (0..8)
        .map(|i| bpsk_bit_mod(((1_u8 << i) & byte).rotate_right(i)).unwrap())
        .collect()
}

/// Modulates a pair of bits to a complex int16 impulse via QPSK
pub fn qpsk_bit_mod(bits: u8) -> Option<Complex<i16>> {
    if bits == 0 {
        Some(Complex::new(1, 1))
    } else if bits == 1 {
        Some(Complex::new(-1, 1))
    } else if bits == 2 {
        Some(Complex::new(1, -1))
    } else if bits == 3 {
        Some(Complex::new(-1, -1))
    } else {
        None
    }
}

/// Modulates a byte via QPSK into 4 int16 samples
pub fn qpsk_byte_mod(byte: u8) -> Vec<Complex<i16>> {
    (0..8)
        .step_by(2)
        .map(|i| qpsk_bit_mod(((3_u8 << i) & byte).rotate_right(i)).unwrap())
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use num::Complex;

    #[test]
    fn test_bpsk_bit() {
        assert_eq!(bpsk_bit_mod(0_u8).unwrap(), Complex::new(1, 0));
        assert_eq!(bpsk_bit_mod(1_u8).unwrap(), Complex::new(-1, 0));
    }

    #[test]
    fn test_qpsk_bit() {
        assert_eq!(qpsk_bit_mod(0_u8).unwrap(), Complex::new(1, 1));
        assert_eq!(qpsk_bit_mod(1_u8).unwrap(), Complex::new(-1, 1));
        assert_eq!(qpsk_bit_mod(2_u8).unwrap(), Complex::new(1, -1));
        assert_eq!(qpsk_bit_mod(3_u8).unwrap(), Complex::new(-1, -1));
    }

    #[test]
    fn test_bpsk_byte() {
        assert_eq!(bpsk_byte_mod(0_u8), vec![Complex::new(1, 0); 8]);
        assert_eq!(
            bpsk_byte_mod(31_u8),
            vec![
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(1, 0),
                Complex::new(1, 0),
                Complex::new(1, 0)
            ]
        );
        assert_eq!(
            bpsk_byte_mod(63_u8),
            vec![
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(1, 0),
                Complex::new(1, 0)
            ]
        );
        assert_eq!(
            bpsk_byte_mod(127_u8),
            vec![
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(-1, 0),
                Complex::new(1, 0)
            ]
        );
        assert_eq!(bpsk_byte_mod(255_u8), vec![Complex::new(-1, 0); 8]);
    }

    #[test]
    fn test_qpsk_byte() {
        assert_eq!(
            qpsk_byte_mod(0_u8),
            vec![
                Complex::new(1, 1),
                Complex::new(1, 1),
                Complex::new(1, 1),
                Complex::new(1, 1)
            ]
        );
        assert_eq!(
            qpsk_byte_mod(2_u8),
            vec![
                Complex::new(1, -1),
                Complex::new(1, 1),
                Complex::new(1, 1),
                Complex::new(1, 1)
            ]
        );
        assert_eq!(
            qpsk_byte_mod(4_u8),
            vec![
                Complex::new(1, 1),
                Complex::new(-1, 1),
                Complex::new(1, 1),
                Complex::new(1, 1)
            ]
        );
        assert_eq!(
            qpsk_byte_mod(15_u8),
            vec![
                Complex::new(-1, -1),
                Complex::new(-1, -1),
                Complex::new(1, 1),
                Complex::new(1, 1)
            ]
        );
        assert_eq!(
            qpsk_byte_mod(254_u8),
            vec![
                Complex::new(1, -1),
                Complex::new(-1, -1),
                Complex::new(-1, -1),
                Complex::new(-1, -1)
            ]
        );
    }
}
