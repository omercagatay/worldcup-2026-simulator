// Smoke test for the argmin 0.11 L-BFGS + Vec backend API.
// Minimizes f(x) = sum_i (x_i - t_i)^2 recovering x = t.

use argmin::{
    core::{CostFunction, Error, Executor, Gradient, State},
    solver::{linesearch::MoreThuenteLineSearch, quasinewton::LBFGS},
};

struct Quad {
    t: Vec<f64>,
}

impl CostFunction for Quad {
    type Param = Vec<f64>;
    type Output = f64;
    fn cost(&self, p: &Self::Param) -> Result<Self::Output, Error> {
        Ok(p.iter()
            .zip(self.t.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum())
    }
}

impl Gradient for Quad {
    type Param = Vec<f64>;
    type Gradient = Vec<f64>;
    fn gradient(&self, p: &Self::Param) -> Result<Self::Gradient, Error> {
        Ok(p.iter()
            .zip(self.t.iter())
            .map(|(a, b)| 2.0 * (a - b))
            .collect())
    }
}

fn main() -> Result<(), Error> {
    let t = vec![1.5_f64, -0.5, 3.0];
    let cost = Quad { t: t.clone() };

    let linesearch = MoreThuenteLineSearch::new().with_c(1e-4, 0.9)?;
    let solver = LBFGS::new(linesearch, 5)
        .with_tolerance_grad(1e-8)?
        .with_tolerance_cost(1e-12)?;

    let init = vec![0.0_f64; t.len()];
    let res = Executor::new(cost, solver)
        .configure(|state| state.param(init).max_iters(100))
        .run()?;

    let best = res.state().get_param().expect("best param");
    for (a, b) in best.iter().zip(t.iter()) {
        println!("{:.6} (target {:.3})", a, b);
        assert!((a - b).abs() < 1e-3, "converged value off target");
    }
    println!("OK");
    Ok(())
}
