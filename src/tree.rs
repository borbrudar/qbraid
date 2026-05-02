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
    pub state: FibState,
    pub braid_prefix: Vec<i32>,
}

fn invert(m: [[Complex64; 2]; 2]) -> [[Complex64; 2]; 2] {
    // unitary inverse = conjugate transpose
    [
        [m[0][0].conj(), m[1][0].conj()],
        [m[0][1].conj(), m[1][1].conj()],
    ]
}

pub fn braid_to_fib_steps(crossings: &[i32]) -> Vec<FibStep> {
    let mut steps = vec![];
    let mut state = FibState::new();

    let mut prefix = vec![];

    for &g in crossings {
        prefix.push(g);

        match g {
            1 => apply_r(&mut steps, &mut state, &prefix),
            -1 => apply_r_inv(&mut steps, &mut state, &prefix),
            2 => {
                apply_f(&mut steps, &mut state, &prefix);
                apply_r(&mut steps, &mut state, &prefix);
                apply_f_inv(&mut steps, &mut state, &prefix);
            }
            -2 => {
                apply_f(&mut steps, &mut state, &prefix);
                apply_r_inv(&mut steps, &mut state, &prefix);
                apply_f_inv(&mut steps, &mut state, &prefix);
            }
            _ => {}
        }
    }

    steps
}

fn apply_f(steps: &mut Vec<FibStep>, state: &mut FibState, prefix: &[i32]) {
    state.vec = matmul(f_matrix(), state.vec);
    state.basis = match state.basis {
        FusionBasis::Left => FusionBasis::Right,
        FusionBasis::Right => FusionBasis::Left,
    };

    steps.push(FibStep {
        op: FibOp::F,
        label: "F".into(),
        state: state.clone(),
        braid_prefix: prefix.to_vec(),
    });
}

fn apply_f_inv(steps: &mut Vec<FibStep>, state: &mut FibState, prefix: &[i32]) {
    state.vec = matmul(invert(f_matrix()), state.vec);
    state.basis = match state.basis {
        FusionBasis::Left => FusionBasis::Right,
        FusionBasis::Right => FusionBasis::Left,
    };

    steps.push(FibStep {
        op: FibOp::FInv,
        label: "F⁻¹".into(),
        state: state.clone(),
        braid_prefix: prefix.to_vec(),
    });
}

fn apply_r(steps: &mut Vec<FibStep>, state: &mut FibState, prefix: &[i32]) {
    // ensure LEFT basis
    if let FusionBasis::Right = state.basis {
        apply_f(steps, state, prefix);
    }

    state.vec = matmul(r_matrix(), state.vec);

    steps.push(FibStep {
        op: FibOp::R,
        label: "R".into(),
        state: state.clone(),
        braid_prefix: prefix.to_vec(),
    });
}

fn apply_r_inv(steps: &mut Vec<FibStep>, state: &mut FibState, prefix: &[i32]) {
    if let FusionBasis::Right = state.basis {
        apply_f(steps, state, prefix);
    }

    state.vec = matmul(invert(r_matrix()), state.vec);

    steps.push(FibStep {
        op: FibOp::RInv,
        label: "R⁻¹".into(),
        state: state.clone(),
        braid_prefix: prefix.to_vec(),
    });
}

pub fn compute_total(steps: &[FibStep]) -> [Complex64; 2] {
    steps
        .last()
        .map(|s| s.state.vec)
        .unwrap_or([Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)])
}
