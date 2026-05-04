use crate::fibonacci::*;
use num_complex::Complex64;

#[derive(Clone, Debug)]
pub struct FibResult {
    pub raw: [[Complex64; 2]; 2],
    pub normalized: [[Complex64; 2]; 2],
}

fn identity() -> [[Complex64; 2]; 2] {
    [
        [Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
        [Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)],
    ]
}

fn matmul(a: [[Complex64; 2]; 2], b: [[Complex64; 2]; 2]) -> [[Complex64; 2]; 2] {
    [
        [
            a[0][0] * b[0][0] + a[0][1] * b[1][0],
            a[0][0] * b[0][1] + a[0][1] * b[1][1],
        ],
        [
            a[1][0] * b[0][0] + a[1][1] * b[1][0],
            a[1][0] * b[0][1] + a[1][1] * b[1][1],
        ],
    ]
}

// remove global phase
fn normalize(m: [[Complex64; 2]; 2]) -> [[Complex64; 2]; 2] {
    let det = m[0][0] * m[1][1] - m[0][1] * m[1][0];

    let phase = (det.arg() * 0.5).cos() + Complex64::i() * (det.arg() * 0.5).sin();
    let inv_phase = phase.conj();

    [
        [m[0][0] * inv_phase, m[0][1] * inv_phase],
        [m[1][0] * inv_phase, m[1][1] * inv_phase],
    ]
}

pub fn evaluate_braid(crossings: &[i32]) -> FibResult {
    let mut U = identity();

    for &g in crossings {
        let op = match g {
            1 => sigma1(),
            -1 => sigma1_inv(),
            2 => sigma2(),
            -2 => sigma2_inv(),
            _ => continue,
        };

        U = matmul(op, U);
    }

    FibResult {
        raw: U,
        normalized: normalize(U),
    }
}
