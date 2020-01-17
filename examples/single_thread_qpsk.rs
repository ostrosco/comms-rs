use byteorder::{NativeEndian, WriteBytesExt};
use comms_rs::filter::fir;
use comms_rs::util::math;
use num::{Complex, Zero};
use rand::distributions::Uniform;
use rand::Rng;
use std::fs::File;
use std::io::BufWriter;
use std::time::Instant;

/// An example that will generate random numbers, pass them through a QPSK
/// modulation and a pulse shaper, then broadcast them out to a file and
/// via ZeroMQ for visualization.
fn main() {
    let mut rng = rand::thread_rng();
    let sam_per_sym = 4.0;
    let taps: Vec<Complex<f32>> =
        math::rrc_taps(32, sam_per_sym, 0.25).unwrap();
    let mut state: Vec<Complex<f32>> = vec![Complex::zero(); 32];
    let mut writer = BufWriter::new(File::create("./qpsk_out.bin").unwrap());
    let now = Instant::now();
    let dist = Uniform::new(0u8, 2u8);

    loop {
        let mut bits: Vec<u8> = Vec::new();
        for _ in 0..4096 {
            bits.push(rng.sample(&dist));
        }
        let qpsk_mod: Vec<Complex<f32>> = bits
            .iter().step_by(2).zip(bits.iter().skip(1).step_by(2))
            .map(|(x, y)| Complex::new(f32::from(*x) * 2.0 - 1.0, *y as f32 * 2.0 - 1.0))
            .collect();
        let mut upsample = vec![Complex::zero(); 4096 * 2];
        let mut ix = 0;
        for samp in qpsk_mod {
            upsample[ix] = samp;
            ix += 4;
        }
        let pulse_shape = fir::batch_fir(&upsample, &taps, &mut state);
        pulse_shape
            .iter()
            .map(|x| Complex::new((8192.0 * x.re) as i16, (8192.0 * x.im) as i16))
            .for_each(|x| {
                writer.write_i16::<NativeEndian>(x.re).unwrap();
                writer.write_i16::<NativeEndian>(x.im).unwrap();
            });
        if now.elapsed().as_secs() >= 10 {
            break;
        }
    }
}
