use num_complex::Complex64;

#[derive(Clone, Copy, Debug)]
pub enum FusionBasis {
    Left,  // (ττ)τ
    Right, // τ(ττ)
}

#[derive(Clone, Debug)]
pub struct FibState {
    pub basis: FusionBasis,
    pub vec: [Complex64; 2], // amplitudes for channels (1, τ)
}

impl FibState {
    pub fn new() -> Self {
        Self {
            basis: FusionBasis::Left,
            vec: [Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
        }
    }
}

pub fn phi() -> f64 {
    (1.0 + 5.0_f64.sqrt()) / 2.0
}

pub fn f_matrix() -> [[Complex64; 2]; 2] {
    let phi = phi();
    let inv_phi = 1.0 / phi;
    let sqrt_inv_phi = inv_phi.sqrt();

    [
        [
            Complex64::new(inv_phi, 0.0),
            Complex64::new(sqrt_inv_phi, 0.0),
        ],
        [
            Complex64::new(sqrt_inv_phi, 0.0),
            Complex64::new(-inv_phi, 0.0),
        ],
    ]
}

pub fn r_matrix() -> [[Complex64; 2]; 2] {
    use std::f64::consts::PI;

    let r1 = Complex64::from_polar(1.0, -4.0 * PI / 5.0);
    let r_tau = Complex64::from_polar(1.0, 3.0 * PI / 5.0);

    [
        [r1, Complex64::new(0.0, 0.0)],
        [Complex64::new(0.0, 0.0), r_tau],
    ]
}

pub fn matmul(a: [[Complex64; 2]; 2], v: [Complex64; 2]) -> [Complex64; 2] {
    [
        a[0][0] * v[0] + a[0][1] * v[1],
        a[1][0] * v[0] + a[1][1] * v[1],
    ]
}
