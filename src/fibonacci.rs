use num_complex::Complex64;

const PI: f64 = std::f64::consts::PI;

fn exp(theta: f64) -> Complex64 {
    Complex64::from_polar(1.0, theta)
}

fn phi() -> f64 {
    (1.0 + 5.0_f64.sqrt()) / 2.0
}

fn sqrt_phi_inv() -> f64 {
    (1.0 / phi()).sqrt()
}

fn invert(m: [[Complex64; 2]; 2]) -> [[Complex64; 2]; 2] {
    [
        [m[0][0].conj(), m[1][0].conj()],
        [m[0][1].conj(), m[1][1].conj()],
    ]
}

// oba braidinga + inverzi
pub fn sigma1() -> [[Complex64; 2]; 2] {
    let e1 = exp(-4.0 * PI / 5.0);
    let e2 = exp(3.0 * PI / 5.0);

    [
        [e1, Complex64::new(0.0, 0.0)],
        [Complex64::new(0.0, 0.0), e2],
    ]
}

pub fn sigma2() -> [[Complex64; 2]; 2] {
    let phi_inv = 1.0 / phi();
    let phi_inv_sqrt = sqrt_phi_inv();

    let a = phi_inv * exp(4.0 * PI / 5.0);
    let b = phi_inv_sqrt * exp(-3.0 * PI / 5.0);
    let c = Complex64::from(-phi_inv);

    [[a, b], [b, c]]
}

pub fn sigma1_inv() -> [[Complex64; 2]; 2] {
    invert(sigma1())
}

pub fn sigma2_inv() -> [[Complex64; 2]; 2] {
    invert(sigma2())
}
