use rand::Rng;
use crate::{Real};

pub(crate) fn generate_pulse(t: &Vec<Real>) -> Vec<Real> {
    let mut offset: Real = 1.0;
    let noise = rand::thread_rng().gen_range(-0.02..0.02);
    offset = offset + noise;
    let mut pulse: Vec<Real> = vec![0.0; t.len()];
    let noise_rng: Vec<Real> = (0..t.len()).map(|_| rand::thread_rng().gen_range(0.0..0.02)).collect();
    for i in 0..t.len() {
        if t[i] > 2.0 {
            pulse[i] = 0.0;
        } else if t[i] < -2.0 {
            pulse[i] = 0.0
        } else {
            pulse[i] = (t[i] + offset).sin().powi(99) * (t[i] + offset + 1.5).sin().powi(8);
        }
        pulse[i] = pulse[i] + noise_rng[i];
    }
    return pulse;
}
