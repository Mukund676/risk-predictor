use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use rand_distr::StandardNormal;
use rayon::prelude::*;

const BASE_DT_EPSILON: f64 = 1.0e-12;
const SAMPLE_PATH_STRIDE_MIN: usize = 1;

#[derive(Debug, Clone, Copy)]
pub struct MonteCarloConfig {
    pub num_paths: usize,
    pub num_steps: usize,
    pub seed: u64,
    pub sample_path_count: usize,
}

impl MonteCarloConfig {
    pub fn new(num_paths: usize, num_steps: usize, seed: u64, sample_path_count: usize) -> Self {
        Self {
            num_paths,
            num_steps,
            seed,
            sample_path_count,
        }
    }

    fn normalized(self) -> Self {
        Self {
            num_paths: self.num_paths.max(1),
            num_steps: self.num_steps.max(1),
            seed: self.seed,
            sample_path_count: self.sample_path_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathSeries {
    pub path_index: usize,
    pub prices: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonteCarloResult {
    pub terminal_pnl_samples: Vec<f64>,
    pub sample_paths: Vec<PathSeries>,
    pub terminal_prices: Vec<f64>,
}

#[derive(Debug, Clone, Copy)]
pub struct GbmRequest {
    pub initial_price: f64,
    pub drift: f64,
    pub volatility: f64,
    pub horizon_years: f64,
    pub initial_portfolio_value: f64,
    pub config: MonteCarloConfig,
}

impl GbmRequest {
    pub fn new(
        initial_price: f64,
        drift: f64,
        volatility: f64,
        horizon_years: f64,
        initial_portfolio_value: f64,
        config: MonteCarloConfig,
    ) -> Self {
        Self {
            initial_price,
            drift,
            volatility,
            horizon_years,
            initial_portfolio_value,
            config,
        }
    }
}

pub fn simulate_portfolio_monte_carlo(request: &GbmRequest) -> MonteCarloResult {
    let config = request.config.normalized();
    let dt = if request.horizon_years.is_finite() && request.horizon_years > 0.0 {
        request.horizon_years / config.num_steps as f64
    } else {
        BASE_DT_EPSILON
    };

    let mut paths: Vec<PathSeries> = (0..config.num_paths)
        .into_par_iter()
        .map(|path_index| {
            let seed = derive_path_seed(request.config.seed, path_index);
            let mut rng = StdRng::seed_from_u64(seed);
            let mut prices = Vec::with_capacity(config.num_steps + 1);
            let mut price = request.initial_price;

            prices.push(price);

            if request.volatility <= 0.0 || dt <= 0.0 || !price.is_finite() || price <= 0.0 {
                for _ in 0..config.num_steps {
                    prices.push(price.max(0.0));
                }
                return PathSeries { path_index, prices };
            }

            let drift_term = (request.drift - 0.5 * request.volatility * request.volatility) * dt;
            let diffusion_scale = request.volatility * dt.sqrt();

            for _ in 0..config.num_steps {
                let z: f64 = rng.sample(StandardNormal);
                price *= (drift_term + diffusion_scale * z).exp();
                if !price.is_finite() || price < 0.0 {
                    price = 0.0;
                }
                prices.push(price);
            }

            PathSeries { path_index, prices }
        })
        .collect();

    paths.sort_unstable_by_key(|path| path.path_index);

    let terminal_prices: Vec<f64> = paths
        .iter()
        .map(|path| path.prices.last().copied().unwrap_or(request.initial_price))
        .collect();

    let terminal_pnl_samples: Vec<f64> = terminal_prices
        .iter()
        .map(|terminal_price| terminal_price - request.initial_portfolio_value)
        .collect();

    let sample_paths = select_sample_paths(&paths, config.sample_path_count);

    MonteCarloResult {
        terminal_pnl_samples,
        sample_paths,
        terminal_prices,
    }
}

pub fn gbm_step(current_price: f64, drift: f64, volatility: f64, dt: f64, z: f64) -> f64 {
    if !current_price.is_finite() || !drift.is_finite() || !volatility.is_finite() || !dt.is_finite() || !z.is_finite() {
        return 0.0;
    }

    if current_price <= 0.0 {
        return 0.0;
    }

    if dt <= 0.0 || volatility <= 0.0 {
        return current_price;
    }

    let exponent = (drift - 0.5 * volatility * volatility) * dt + volatility * dt.sqrt() * z;
    let next_price = current_price * exponent.exp();

    if next_price.is_finite() && next_price >= 0.0 {
        next_price
    } else {
        0.0
    }
}

pub fn generate_price_path(
    initial_price: f64,
    drift: f64,
    volatility: f64,
    horizon_years: f64,
    num_steps: usize,
    seed: u64,
) -> PathSeries {
    let config = MonteCarloConfig::new(1, num_steps, seed, 1).normalized();
    let dt = if horizon_years.is_finite() && horizon_years > 0.0 {
        horizon_years / config.num_steps as f64
    } else {
        BASE_DT_EPSILON
    };

    let mut rng = StdRng::seed_from_u64(derive_path_seed(seed, 0));
    let mut prices = Vec::with_capacity(config.num_steps + 1);
    let mut price = initial_price;

    prices.push(price);

    for _ in 0..config.num_steps {
        let z: f64 = rng.sample(StandardNormal);
        price = gbm_step(price, drift, volatility, dt, z);
        prices.push(price);
    }

    PathSeries {
        path_index: 0,
        prices,
    }
}

fn derive_path_seed(base_seed: u64, path_index: usize) -> u64 {
    let mixed = base_seed ^ ((path_index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    mixed.wrapping_add(0xD2B7_4407_B1CE_6E93)
}

fn select_sample_paths(paths: &[PathSeries], sample_count: usize) -> Vec<PathSeries> {
    if paths.is_empty() {
        return Vec::new();
    }

    let target_count = sample_count.max(SAMPLE_PATH_STRIDE_MIN).min(paths.len());

    if target_count == paths.len() {
        return paths.to_vec();
    }

    let mut selected = Vec::with_capacity(target_count);
    for sample_index in 0..target_count {
        let source_index = sample_index * paths.len() / target_count;
        selected.push(paths[source_index].clone());
    }
    selected
}

#[cfg(test)]
mod tests {
    use super::{gbm_step, generate_price_path, simulate_portfolio_monte_carlo, GbmRequest, MonteCarloConfig};

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        let difference = (actual - expected).abs();
        assert!(
            difference <= tolerance,
            "expected {expected}, got {actual}, difference {difference} exceeded tolerance {tolerance}"
        );
    }

    #[test]
    fn gbm_drift_calculation_matches_closed_form_over_one_step() {
        let current_price = 100.0;
        let drift = 0.08;
        let volatility = 0.2;
        let dt = 1.0 / 252.0;
        let z = 0.0;

        let expected = current_price * ((drift - 0.5_f64 * volatility * volatility) * dt).exp();
        let actual = gbm_step(current_price, drift, volatility, dt, z);

        assert_close(actual, expected, 1e-12);
    }

    #[test]
    fn identical_rng_seed_produces_identical_simulation_output() {
        let config = MonteCarloConfig::new(128, 32, 7_777, 8);
        let request = GbmRequest::new(100.0, 0.06, 0.24, 1.0, 100.0, config);

        let left = simulate_portfolio_monte_carlo(&request);
        let right = simulate_portfolio_monte_carlo(&request);

        assert_eq!(left.terminal_pnl_samples, right.terminal_pnl_samples);
        assert_eq!(left.terminal_prices, right.terminal_prices);
        assert_eq!(left.sample_paths, right.sample_paths);
    }

    #[test]
    fn generated_path_has_expected_shape_and_is_reproducible() {
        let left = generate_price_path(100.0, 0.05, 0.2, 1.0, 16, 42);
        let right = generate_price_path(100.0, 0.05, 0.2, 1.0, 16, 42);

        assert_eq!(left, right);
        assert_eq!(left.prices.len(), 17);
        assert_eq!(left.prices[0], 100.0);
    }
}
