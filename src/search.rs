use crate::fibonacci::*;
use num_complex::Complex64;
use rand::Rng;
use std::f64::consts::PI;

type Mat2 = [[Complex64; 2]; 2];

fn identity() -> Mat2 {
    [
        [Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
        [Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)],
    ]
}

fn matmul(a: Mat2, b: Mat2) -> Mat2 {
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


fn phase_invariant_distance(a: Mat2, b: Mat2) -> f64 {
    // find best global phase alignment using trace overlap
    let trace = a[0][0] * b[0][0].conj()
        + a[0][1] * b[0][1].conj()
        + a[1][0] * b[1][0].conj()
        + a[1][1] * b[1][1].conj();

    let phase = trace / trace.norm().max(1e-12);

    let b_aligned = [
        [b[0][0] * phase, b[0][1] * phase],
        [b[1][0] * phase, b[1][1] * phase],
    ];

    // Frobenius norm
    let mut sum = 0.0;
    for i in 0..2 {
        for j in 0..2 {
            let d = a[i][j] - b_aligned[i][j];
            sum += d.norm_sqr();
        }
    }

    sum.sqrt()
}

pub fn random_unitary() -> Mat2 {
    let u1: f64 = rand::random();
    let u2: f64 = rand::random();
    let u3: f64 = rand::random();

    let theta = (1.0 - u1).sqrt().acos();
    let phi = 2.0 * PI * u2;
    let psi = 2.0 * PI * u3;

    let a = Complex64::from_polar(theta.cos(), phi);
    let b = Complex64::from_polar(theta.sin(), psi);

    // SU(2) parameterization
    [
        [a, b],
        [-b.conj(), a.conj()],
    ]
}

// braid eval

fn apply_generator(u: Mat2, g: i32) -> Mat2 {
    let op = match g {
        1 => sigma1(),
        -1 => sigma1_inv(),
        2 => sigma2(),
        -2 => sigma2_inv(),
        _ => identity(),
    };

    matmul(op, u)
}

fn evaluate_word(word: &[i32]) -> Mat2 {
    let mut u = identity();
    for &g in word {
        u = apply_generator(u, g);
    }
    u
}


pub fn brute_force_approx(
    target: Mat2,
    max_depth: usize,
    alphabet: &[i32], // e.g. &[1, -1, 2, -2]
) -> (Vec<i32>, f64, Mat2) {
    let mut best_word = vec![];
    let mut best_dist = f64::INFINITY;
    let mut best_mat = identity();

    fn dfs(
        depth: usize,
        max_depth: usize,
        current: &mut Vec<i32>,
        alphabet: &[i32],
        target: Mat2,
        best_word: &mut Vec<i32>,
        best_dist: &mut f64,
        best_mat: &mut Mat2,
    ) {
        if depth == max_depth {
            let u = evaluate_word(current);
            let d = phase_invariant_distance(u, target);

            if d < *best_dist {
                *best_dist = d;
                *best_word = current.clone();
                *best_mat = u;
            }
            return;
        }

        for &g in alphabet {
            current.push(g);
            dfs(
                depth + 1,
                max_depth,
                current,
                alphabet,
                target,
                best_word,
                best_dist,
                best_mat,
            );
            current.pop();
        }
    }

    let mut current = vec![];

    for d in 1..=max_depth {
        dfs(
            0,
            d,
            &mut current,
            alphabet,
            target,
            &mut best_word,
            &mut best_dist,
            &mut best_mat,
        );
    }

    (best_word, best_dist, best_mat)
}

pub struct SearchResult {
    pub word: Vec<i32>,
    pub distance: f64,
}

pub fn find_braid(
    target: Mat2,
    depth: usize,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<SearchResult, String> {
    let alphabet = &[1, -1, 2, -2];

    let (word, dist, _mat) = brute_force_approx(target, depth, alphabet);

    if stop.load(std::sync::atomic::Ordering::Relaxed) {
        return Err("Search stopped".into());
    }

    Ok(SearchResult { word, distance: dist })
}