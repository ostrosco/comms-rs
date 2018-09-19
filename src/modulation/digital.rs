//! Provide tools to do digital modulation

extern crate num_complex;

/// Modulates a bit to a complex int16 impulse via BPSK
fn bpsk_bit_mod(bit: u8) -> Option<num_complex::Complex<i16>> {
    if bit == 1 {
        Some(num_complex::Complex::new(1, 0))
    } else if bit == 0 {
        Some(num_complex::Complex::new(-1, 0))
    }
    else {
        None
    }
}

/// Modulates a byte via BPSK into int16 samples
fn bpsk_byte_mod(byte: u8) -> Vec<num_complex::Complex<i16>> {
    let mut mod_data = Vec::with_capacity(8);
    let first_bit = 0b0001 & byte;
    let second_bit = (0b0010 & byte).rotate_right(1);
    let third_bit =  (0b0100 & byte).rotate_right(2);
    let fourth_bit = (0b1000 & byte).rotate_right(3);

    // We know that this can't be greater than 1, because of explicit bit twiddling
    mod_data[0] = bpsk_bit_mod(first_bit).unwrap();
    mod_data[1] = bpsk_bit_mod(second_bit).unwrap();
    mod_data[2] = bpsk_bit_mod(third_bit).unwrap();
    mod_data[3] = bpsk_bit_mod(fourth_bit).unwrap();

    mod_data
}

