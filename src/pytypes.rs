use pyo3::prelude::*;

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct EquityInstrument {
    #[pyo3(get, set)]
    pub instrument_id: String,
    #[pyo3(get, set)]
    pub symbol: String,
    #[pyo3(get, set)]
    pub exchange: Option<String>,
    #[pyo3(get, set)]
    pub currency: String,
}

#[pymethods]
impl EquityInstrument {
    #[new]
    #[pyo3(signature = (instrument_id, symbol, exchange, currency))]
    fn new(instrument_id: String, symbol: String, exchange: Option<String>, currency: String) -> Self {
        Self {
            instrument_id,
            symbol,
            exchange,
            currency,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct EuropeanOptionInstrument {
    #[pyo3(get, set)]
    pub instrument_id: String,
    #[pyo3(get, set)]
    pub underlying_symbol: String,
    #[pyo3(get, set)]
    pub symbol: Option<String>,
    #[pyo3(get, set)]
    pub strike: f64,
    #[pyo3(get, set)]
    pub expiration_date: String,
    #[pyo3(get, set)]
    pub style: String,
    #[pyo3(get, set)]
    pub multiplier: f64,
    #[pyo3(get, set)]
    pub currency: String,
    #[pyo3(get, set)]
    pub exchange: Option<String>,
}

#[pymethods]
impl EuropeanOptionInstrument {
    #[new]
    #[pyo3(signature = (instrument_id, underlying_symbol, symbol, strike, expiration_date, style, multiplier, currency, exchange))]
    fn new(
        instrument_id: String,
        underlying_symbol: String,
        symbol: Option<String>,
        strike: f64,
        expiration_date: String,
        style: String,
        multiplier: f64,
        currency: String,
        exchange: Option<String>,
    ) -> Self {
        Self {
            instrument_id,
            underlying_symbol,
            symbol,
            strike,
            expiration_date,
            style,
            multiplier,
            currency,
            exchange,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct PortfolioPosition {
    #[pyo3(get, set)]
    pub position_id: String,
    #[pyo3(get, set)]
    pub instrument: PyObject,
    #[pyo3(get, set)]
    pub quantity: f64,
    #[pyo3(get, set)]
    pub average_cost_basis: Option<f64>,
    #[pyo3(get, set)]
    pub acquisition_date: Option<String>,
}

#[pymethods]
impl PortfolioPosition {
    #[new]
    fn new(
        position_id: String,
        instrument: PyObject,
        quantity: f64,
        average_cost_basis: Option<f64>,
        acquisition_date: Option<String>,
    ) -> Self {
        Self {
            position_id,
            instrument,
            quantity,
            average_cost_basis,
            acquisition_date,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct Portfolio {
    #[pyo3(get, set)]
    pub portfolio_id: String,
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub base_currency: String,
    #[pyo3(get, set)]
    pub as_of: String,
    #[pyo3(get, set)]
    pub positions: Vec<Py<PortfolioPosition>>,
}

#[pymethods]
impl Portfolio {
    #[new]
    fn new(
        portfolio_id: String,
        name: String,
        base_currency: String,
        as_of: String,
        positions: Vec<Py<PortfolioPosition>>,
    ) -> Self {
        Self {
            portfolio_id,
            name,
            base_currency,
            as_of,
            positions,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct MarketQuote {
    #[pyo3(get, set)]
    pub symbol: String,
    #[pyo3(get, set)]
    pub spot_price: f64,
    #[pyo3(get, set)]
    pub currency: String,
    #[pyo3(get, set)]
    pub last_updated_at: Option<String>,
}

#[pymethods]
impl MarketQuote {
    #[new]
    fn new(symbol: String, spot_price: f64, currency: String, last_updated_at: Option<String>) -> Self {
        Self {
            symbol,
            spot_price,
            currency,
            last_updated_at,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct DividendYieldPoint {
    #[pyo3(get, set)]
    pub symbol: String,
    #[pyo3(get, set)]
    pub dividend_yield: f64,
}

#[pymethods]
impl DividendYieldPoint {
    #[new]
    fn new(symbol: String, dividend_yield: f64) -> Self {
        Self { symbol, dividend_yield }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct VolatilityPoint {
    #[pyo3(get, set)]
    pub symbol: String,
    #[pyo3(get, set)]
    pub historical_volatility: f64,
}

#[pymethods]
impl VolatilityPoint {
    #[new]
    fn new(symbol: String, historical_volatility: f64) -> Self {
        Self {
            symbol,
            historical_volatility,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct CorrelationMatrix {
    #[pyo3(get, set)]
    pub symbols: Vec<String>,
    #[pyo3(get, set)]
    pub values: Vec<Vec<f64>>,
}

#[pymethods]
impl CorrelationMatrix {
    #[new]
    fn new(symbols: Vec<String>, values: Vec<Vec<f64>>) -> Self {
        Self { symbols, values }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct MarketEnvironment {
    #[pyo3(get, set)]
    pub as_of: String,
    #[pyo3(get, set)]
    pub risk_free_rate: f64,
    #[pyo3(get, set)]
    pub dividend_yields: Vec<Py<DividendYieldPoint>>,
    #[pyo3(get, set)]
    pub quotes: Vec<Py<MarketQuote>>,
    #[pyo3(get, set)]
    pub volatilities: Vec<Py<VolatilityPoint>>,
    #[pyo3(get, set)]
    pub correlation_matrix: Option<Py<CorrelationMatrix>>,
}

#[pymethods]
impl MarketEnvironment {
    #[new]
    fn new(
        as_of: String,
        risk_free_rate: f64,
        dividend_yields: Vec<Py<DividendYieldPoint>>,
        quotes: Vec<Py<MarketQuote>>,
        volatilities: Vec<Py<VolatilityPoint>>,
        correlation_matrix: Option<Py<CorrelationMatrix>>,
    ) -> Self {
        Self {
            as_of,
            risk_free_rate,
            dividend_yields,
            quotes,
            volatilities,
            correlation_matrix,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct EquityShock {
    #[pyo3(get, set)]
    pub symbol: String,
    #[pyo3(get, set)]
    pub expected_return_shift_pct: f64,
    #[pyo3(get, set)]
    pub volatility_shift_pct: f64,
}

#[pymethods]
impl EquityShock {
    #[new]
    fn new(symbol: String, expected_return_shift_pct: f64, volatility_shift_pct: f64) -> Self {
        Self {
            symbol,
            expected_return_shift_pct,
            volatility_shift_pct,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct SpotShock {
    #[pyo3(get, set)]
    pub symbol: String,
    #[pyo3(get, set)]
    pub spot_return_shift_pct: f64,
}

#[pymethods]
impl SpotShock {
    #[new]
    fn new(symbol: String, spot_return_shift_pct: f64) -> Self {
        Self {
            symbol,
            spot_return_shift_pct,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct ScenarioSpec {
    #[pyo3(get, set)]
    pub scenario_id: String,
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub horizon_days: u32,
    #[pyo3(get, set)]
    pub rate_shift_bps: f64,
    #[pyo3(get, set)]
    pub volatility_shift_pct: f64,
    #[pyo3(get, set)]
    pub correlation_scale: f64,
    #[pyo3(get, set)]
    pub equity_shocks: Vec<Py<EquityShock>>,
    #[pyo3(get, set)]
    pub custom_spot_shocks: Vec<Py<SpotShock>>,
    #[pyo3(get, set)]
    pub notes: Option<String>,
}

#[pymethods]
impl ScenarioSpec {
    #[new]
    fn new(
        scenario_id: String,
        name: String,
        horizon_days: u32,
        rate_shift_bps: f64,
        volatility_shift_pct: f64,
        correlation_scale: f64,
        equity_shocks: Vec<Py<EquityShock>>,
        custom_spot_shocks: Vec<Py<SpotShock>>,
        notes: Option<String>,
    ) -> Self {
        Self {
            scenario_id,
            name,
            horizon_days,
            rate_shift_bps,
            volatility_shift_pct,
            correlation_scale,
            equity_shocks,
            custom_spot_shocks,
            notes,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct MonteCarloConfig {
    #[pyo3(get, set)]
    pub num_paths: u32,
    #[pyo3(get, set)]
    pub num_steps: u32,
    #[pyo3(get, set)]
    pub seed: u64,
    #[pyo3(get, set)]
    pub use_antithetic_variates: bool,
    #[pyo3(get, set)]
    pub use_correlated_draws: bool,
    #[pyo3(get, set)]
    pub sample_path_count: u32,
}

#[pymethods]
impl MonteCarloConfig {
    #[new]
    fn new(
        num_paths: u32,
        num_steps: u32,
        seed: u64,
        use_antithetic_variates: bool,
        use_correlated_draws: bool,
        sample_path_count: u32,
    ) -> Self {
        Self {
            num_paths,
            num_steps,
            seed,
            use_antithetic_variates,
            use_correlated_draws,
            sample_path_count,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct RiskConfig {
    #[pyo3(get, set)]
    pub var_levels: Vec<f64>,
    #[pyo3(get, set)]
    pub compute_cvar: bool,
    #[pyo3(get, set)]
    pub cvar_tail_levels: Vec<f64>,
    #[pyo3(get, set)]
    pub compute_position_greeks: bool,
}

#[pymethods]
impl RiskConfig {
    #[new]
    fn new(
        var_levels: Vec<f64>,
        compute_cvar: bool,
        cvar_tail_levels: Vec<f64>,
        compute_position_greeks: bool,
    ) -> Self {
        Self {
            var_levels,
            compute_cvar,
            cvar_tail_levels,
            compute_position_greeks,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct PortfolioRequest {
    #[pyo3(get, set)]
    pub request_id: String,
    #[pyo3(get, set)]
    pub portfolio: Py<Portfolio>,
    #[pyo3(get, set)]
    pub market: Py<MarketEnvironment>,
    #[pyo3(get, set)]
    pub scenario: Py<ScenarioSpec>,
    #[pyo3(get, set)]
    pub monte_carlo: Py<MonteCarloConfig>,
    #[pyo3(get, set)]
    pub risk_config: Py<RiskConfig>,
    #[pyo3(get, set)]
    pub compute_greeks: bool,
    #[pyo3(get, set)]
    pub compute_path_samples: bool,
}

#[pymethods]
impl PortfolioRequest {
    #[new]
    fn new(
        request_id: String,
        portfolio: Py<Portfolio>,
        market: Py<MarketEnvironment>,
        scenario: Py<ScenarioSpec>,
        monte_carlo: Py<MonteCarloConfig>,
        risk_config: Py<RiskConfig>,
        compute_greeks: bool,
        compute_path_samples: bool,
    ) -> Self {
        Self {
            request_id,
            portfolio,
            market,
            scenario,
            monte_carlo,
            risk_config,
            compute_greeks,
            compute_path_samples,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct PortfolioValuation {
    #[pyo3(get, set)]
    pub total_market_value: f64,
    #[pyo3(get, set)]
    pub total_pnl: f64,
    #[pyo3(get, set)]
    pub base_currency: String,
}

#[pymethods]
impl PortfolioValuation {
    #[new]
    fn new(total_market_value: f64, total_pnl: f64, base_currency: String) -> Self {
        Self {
            total_market_value,
            total_pnl,
            base_currency,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct GreekVector {
    #[pyo3(get, set)]
    pub delta: f64,
    #[pyo3(get, set)]
    pub gamma: f64,
    #[pyo3(get, set)]
    pub vega: f64,
}

#[pymethods]
impl GreekVector {
    #[new]
    fn new(delta: f64, gamma: f64, vega: f64) -> Self {
        Self { delta, gamma, vega }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct RiskMeasures {
    #[pyo3(get, set)]
    pub var_95: f64,
    #[pyo3(get, set)]
    pub var_99: f64,
    #[pyo3(get, set)]
    pub cvar_95: Option<f64>,
    #[pyo3(get, set)]
    pub cvar_99: Option<f64>,
}

#[pymethods]
impl RiskMeasures {
    #[new]
    fn new(var_95: f64, var_99: f64, cvar_95: Option<f64>, cvar_99: Option<f64>) -> Self {
        Self {
            var_95,
            var_99,
            cvar_95,
            cvar_99,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct PositionRiskResult {
    #[pyo3(get, set)]
    pub position_id: String,
    #[pyo3(get, set)]
    pub instrument: PyObject,
    #[pyo3(get, set)]
    pub fair_value: f64,
    #[pyo3(get, set)]
    pub pnl: f64,
    #[pyo3(get, set)]
    pub greeks: Option<Py<GreekVector>>,
}

#[pymethods]
impl PositionRiskResult {
    #[new]
    fn new(
        position_id: String,
        instrument: PyObject,
        fair_value: f64,
        pnl: f64,
        greeks: Option<Py<GreekVector>>,
    ) -> Self {
        Self {
            position_id,
            instrument,
            fair_value,
            pnl,
            greeks,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct PathSeries {
    #[pyo3(get, set)]
    pub path_index: u32,
    #[pyo3(get, set)]
    pub prices: Vec<f64>,
}

#[pymethods]
impl PathSeries {
    #[new]
    fn new(path_index: u32, prices: Vec<f64>) -> Self {
        Self { path_index, prices }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct SimulationSummary {
    #[pyo3(get, set)]
    pub terminal_pnl_samples: Vec<f64>,
    #[pyo3(get, set)]
    pub sample_paths: Vec<Py<PathSeries>>,
    #[pyo3(get, set)]
    pub histogram_bins: Vec<f64>,
    #[pyo3(get, set)]
    pub histogram_counts: Vec<u32>,
}

#[pymethods]
impl SimulationSummary {
    #[new]
    fn new(
        terminal_pnl_samples: Vec<f64>,
        sample_paths: Vec<Py<PathSeries>>,
        histogram_bins: Vec<f64>,
        histogram_counts: Vec<u32>,
    ) -> Self {
        Self {
            terminal_pnl_samples,
            sample_paths,
            histogram_bins,
            histogram_counts,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct ResultMetadata {
    #[pyo3(get, set)]
    pub generated_at: String,
    #[pyo3(get, set)]
    pub engine_version: String,
    #[pyo3(get, set)]
    pub rng_seed: u64,
    #[pyo3(get, set)]
    pub warnings: Vec<String>,
}

#[pymethods]
impl ResultMetadata {
    #[new]
    fn new(generated_at: String, engine_version: String, rng_seed: u64, warnings: Vec<String>) -> Self {
        Self {
            generated_at,
            engine_version,
            rng_seed,
            warnings,
        }
    }
}

#[pyclass(module = "risk_predictor_core")]
#[derive(Clone)]
pub struct PortfolioRiskResponse {
    #[pyo3(get, set)]
    pub request_id: String,
    #[pyo3(get, set)]
    pub portfolio_id: String,
    #[pyo3(get, set)]
    pub valuation: Py<PortfolioValuation>,
    #[pyo3(get, set)]
    pub aggregate_greeks: Py<GreekVector>,
    #[pyo3(get, set)]
    pub risk_measures: Py<RiskMeasures>,
    #[pyo3(get, set)]
    pub position_results: Vec<Py<PositionRiskResult>>,
    #[pyo3(get, set)]
    pub simulation: Py<SimulationSummary>,
    #[pyo3(get, set)]
    pub metadata: Py<ResultMetadata>,
}

#[pymethods]
impl PortfolioRiskResponse {
    #[new]
    fn new(
        request_id: String,
        portfolio_id: String,
        valuation: Py<PortfolioValuation>,
        aggregate_greeks: Py<GreekVector>,
        risk_measures: Py<RiskMeasures>,
        position_results: Vec<Py<PositionRiskResult>>,
        simulation: Py<SimulationSummary>,
        metadata: Py<ResultMetadata>,
    ) -> Self {
        Self {
            request_id,
            portfolio_id,
            valuation,
            aggregate_greeks,
            risk_measures,
            position_results,
            simulation,
            metadata,
        }
    }
}
