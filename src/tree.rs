use crate::fibonacci::*;
use num_complex::Complex64;

// =========================
// TYPES
// =========================

#[derive(Clone)]
pub enum FibOp {
    Sigma1,
    Sigma1Inv,
    Sigma2,
    Sigma2Inv,
}

#[derive(Clone)]
pub struct FibStep {
    pub op: FibOp,
    pub label: String,

    pub matrix: [[Complex64; 2]; 2], // cumulative unitary

    pub braid_remaining: Vec<i32>, // UI-only (visualization)
}

// =========================
// MATRIX HELPERS
// =========================

fn identity() -> [[Complex64; 2]; 2] {
    [
        [Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
        [Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)],
    ]
}

fn invert(m: [[Complex64; 2]; 2]) -> [[Complex64; 2]; 2] {
    [
        [m[0][0].conj(), m[1][0].conj()],
        [m[0][1].conj(), m[1][1].conj()],
    ]
}

fn matmul(
    a: [[Complex64; 2]; 2],
    b: [[Complex64; 2]; 2],
) -> [[Complex64; 2]; 2] {
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

// =========================
// BRAID GENERATORS
// =========================

fn sigma1() -> [[Complex64; 2]; 2] {
    r_matrix()
}

fn sigma1_inv() -> [[Complex64; 2]; 2] {
    invert(sigma1())
}

// σ₂ = F⁻¹ R F
fn sigma2() -> [[Complex64; 2]; 2] {
    let f = f_matrix();
    let r = r_matrix();
    matmul(matmul(invert(f), r), f)
}

fn sigma2_inv() -> [[Complex64; 2]; 2] {
    invert(sigma2())
}

// =========================
// MAIN PIPELINE
// =========================

pub fn braid_to_fib_steps(crossings: &[i32]) -> Vec<FibStep> {
    let mut steps = Vec::new();

    let mut U = identity();

    // IMPORTANT:
    // forward multiplication = left action
    for (i, &g) in crossings.iter().enumerate() {
        let op = match g {
            1 => sigma1(),
            -1 => sigma1_inv(),
            2 => sigma2(),
            -2 => sigma2_inv(),
            _ => continue,
        };

        // U = op * U
        U = matmul(op, U);

        steps.push(FibStep {
            op: match g {
                1 => FibOp::Sigma1,
                -1 => FibOp::Sigma1Inv,
                2 => FibOp::Sigma2,
                -2 => FibOp::Sigma2Inv,
                _ => unreachable!(),
            },
            label: format!("σ{}", g),
            matrix: U,
            braid_remaining: crossings[i + 1..].to_vec(),
        });
    }

    // final “identity view” padding (UI convenience)
    for _ in 0..3 {
        steps.push(FibStep {
            op: FibOp::Sigma1,
            label: "end".into(),
            matrix: U,
            braid_remaining: vec![],
        });
    }

    steps
}

// =========================
// FINAL RESULT
// =========================

pub fn compute_total(steps: &[FibStep]) -> [[Complex64; 2]; 2] {
    steps.last().map(|s| s.matrix).unwrap_or(identity())
}