use crate::fibonacci::*;
use num_complex::Complex64;

#[derive(Clone)]
pub enum FibOp {
    F,
    FInv,
    R,
    RInv,
}

#[derive(Clone)]
pub struct FibStep {
    pub op: FibOp,
    pub label: String,

    pub state: FibState,             // for fusion tree drawing
    pub matrix: [[Complex64; 2]; 2], // full operator up to this step

    pub braid_remaining: Vec<i32>, // what braid is still visible
}

// =========================
// BASIC MATRIX UTILITIES
// =========================

fn identity() -> [[Complex64; 2]; 2] {
    [
        [Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
        [Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)],
    ]
}

fn invert(m: [[Complex64; 2]; 2]) -> [[Complex64; 2]; 2] {
    // unitary inverse = conjugate transpose
    [
        [m[0][0].conj(), m[1][0].conj()],
        [m[0][1].conj(), m[1][1].conj()],
    ]
}

// =========================
// MAIN PIPELINE
// =========================

pub fn braid_to_fib_steps(crossings: &[i32]) -> Vec<FibStep> {
    let mut steps = vec![];

    let mut state = FibState::new();
    let mut U = identity();

    let mut remaining = crossings.to_vec();

    // process braid from RIGHT → LEFT (unweaving)
    for g in crossings.iter().rev() {
        match g {
            // σ₁
            1 => apply_r_inv(&mut steps, &mut state, &mut U, &remaining),

            // σ₁⁻¹
            -1 => apply_r(&mut steps, &mut state, &mut U, &remaining),

            // σ₂
            2 => {
                apply_f(&mut steps, &mut state, &mut U, &remaining);
                apply_r_inv(&mut steps, &mut state, &mut U, &remaining);
                apply_f_inv(&mut steps, &mut state, &mut U, &remaining);
            }

            // σ₂⁻¹
            -2 => {
                apply_f(&mut steps, &mut state, &mut U, &remaining);
                apply_r(&mut steps, &mut state, &mut U, &remaining);
                apply_f_inv(&mut steps, &mut state, &mut U, &remaining);
            }

            _ => {}
        }

        // visually remove last crossing (unwind)
        remaining.pop();
    }

    // add a few "empty braid" steps so final configuration is visible
    for _ in 0..3 {
        steps.push(FibStep {
            op: FibOp::F,
            label: "unwound".into(),
            state: state.clone(),
            matrix: U,
            braid_remaining: vec![],
        });
    }

    steps
}

// =========================
// OPERATIONS
// =========================

fn apply_f(
    steps: &mut Vec<FibStep>,
    state: &mut FibState,
    U: &mut [[Complex64; 2]; 2],
    remaining: &[i32],
) {
    let F = f_matrix();

    state.vec = matmulv(F, state.vec);
    *U = matmul(F, *U);

    state.basis = match state.basis {
        FusionBasis::Left => FusionBasis::Right,
        FusionBasis::Right => FusionBasis::Left,
    };

    steps.push(FibStep {
        op: FibOp::F,
        label: "F".into(),
        state: state.clone(),
        matrix: *U,
        braid_remaining: remaining.to_vec(),
    });
}

fn apply_f_inv(
    steps: &mut Vec<FibStep>,
    state: &mut FibState,
    U: &mut [[Complex64; 2]; 2],
    remaining: &[i32],
) {
    let F_inv = invert(f_matrix());

    state.vec = matmulv(F_inv, state.vec);
    *U = matmul(F_inv, *U);

    state.basis = match state.basis {
        FusionBasis::Left => FusionBasis::Right,
        FusionBasis::Right => FusionBasis::Left,
    };

    steps.push(FibStep {
        op: FibOp::FInv,
        label: "F⁻¹".into(),
        state: state.clone(),
        matrix: *U,
        braid_remaining: remaining.to_vec(),
    });
}

fn apply_r(
    steps: &mut Vec<FibStep>,
    state: &mut FibState,
    U: &mut [[Complex64; 2]; 2],
    remaining: &[i32],
) {
    // ensure LEFT basis before applying R
    if let FusionBasis::Right = state.basis {
        apply_f(steps, state, U, remaining);
    }

    let R = r_matrix();

    state.vec = matmulv(R, state.vec);
    *U = matmul(R, *U);

    steps.push(FibStep {
        op: FibOp::R,
        label: "R".into(),
        state: state.clone(),
        matrix: *U,
        braid_remaining: remaining.to_vec(),
    });
}

fn apply_r_inv(
    steps: &mut Vec<FibStep>,
    state: &mut FibState,
    U: &mut [[Complex64; 2]; 2],
    remaining: &[i32],
) {
    if let FusionBasis::Right = state.basis {
        apply_f(steps, state, U, remaining);
    }

    let R_inv = invert(r_matrix());

    state.vec = matmulv(R_inv, state.vec);
    *U = matmul(R_inv, *U);

    steps.push(FibStep {
        op: FibOp::RInv,
        label: "R⁻¹".into(),
        state: state.clone(),
        matrix: *U,
        braid_remaining: remaining.to_vec(),
    });
}

// =========================
// FINAL RESULT
// =========================

pub fn compute_total(steps: &[FibStep]) -> [[Complex64; 2]; 2] {
    steps.last().map(|s| s.matrix).unwrap_or(identity())
}
