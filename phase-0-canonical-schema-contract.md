# Phase 0 Canonical Schema Contract

## Purpose
These schemas define the exact boundary between the Python orchestration layer and the Rust quantitative engine. They are intentionally minimal, explicit, and stable so both runtimes can serialize and deserialize the same payloads without interpretation drift.

## Contract Rules
- All timestamps use UTC RFC3339 strings.
- All dates use `YYYY-MM-DD` strings.
- Rates and volatilities are annualized decimals, not percentages, unless a field name explicitly says otherwise.
- Monetary values are expressed in the portfolio base currency and use IEEE-754 `f64` / Python `float` for the first release.
- Long positions use positive quantities; short positions use negative quantities.
- Option multipliers default to `100.0` unless a contract states otherwise.
- The Rust layer owns all pricing and risk math; Python only validates, routes, caches, and narrates.

## Canonical Types

### Shared Enums

#### `InstrumentType`
- `equity`
- `european_call_option`
- `european_put_option`

#### `OptionStyle`
- `european`

#### `ProviderKind`
- `alpha_vantage`
- `financial_modeling_prep`
- `yahoo_finance`
- `local_cache`
- `hosted_llm`
- `local_llm`

## Rust Struct Definitions

```rust
use serde::{Deserialize, Serialize};

pub type RequestId = String;
pub type PortfolioId = String;
pub type PositionId = String;
pub type InstrumentId = String;
pub type Symbol = String;
pub type IsoDate = String;
pub type IsoDateTime = String;
pub type CurrencyCode = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRequest {
    pub request_id: RequestId,
    pub portfolio: Portfolio,
    pub market: MarketEnvironment,
    pub scenario: ScenarioSpec,
    pub monte_carlo: MonteCarloConfig,
    pub risk_config: RiskConfig,
    pub compute_greeks: bool,
    pub compute_path_samples: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub portfolio_id: PortfolioId,
    pub name: String,
    pub base_currency: CurrencyCode,
    pub as_of: IsoDateTime,
    pub positions: Vec<PortfolioPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPosition {
    pub position_id: PositionId,
    pub instrument: Instrument,
    pub quantity: f64,
    pub average_cost_basis: Option<f64>,
    pub acquisition_date: Option<IsoDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "instrument_type", rename_all = "snake_case")]
pub enum Instrument {
    Equity(EquityInstrument),
    EuropeanCallOption(EuropeanOptionInstrument),
    EuropeanPutOption(EuropeanOptionInstrument),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityInstrument {
    pub instrument_id: InstrumentId,
    pub symbol: Symbol,
    pub exchange: Option<String>,
    pub currency: CurrencyCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EuropeanOptionInstrument {
    pub instrument_id: InstrumentId,
    pub underlying_symbol: Symbol,
    pub symbol: Option<Symbol>,
    pub strike: f64,
    pub expiration_date: IsoDate,
    pub style: OptionStyle,
    pub multiplier: f64,
    pub currency: CurrencyCode,
    pub exchange: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEnvironment {
    pub as_of: IsoDateTime,
    pub risk_free_rate: f64,
    pub dividend_yields: Vec<DividendYieldPoint>,
    pub quotes: Vec<MarketQuote>,
    pub volatilities: Vec<VolatilityPoint>,
    pub correlation_matrix: Option<CorrelationMatrix>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketQuote {
    pub symbol: Symbol,
    pub spot_price: f64,
    pub currency: CurrencyCode,
    pub last_updated_at: Option<IsoDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DividendYieldPoint {
    pub symbol: Symbol,
    pub dividend_yield: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityPoint {
    pub symbol: Symbol,
    pub historical_volatility: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    pub symbols: Vec<Symbol>,
    pub values: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioSpec {
    pub scenario_id: String,
    pub name: String,
    pub horizon_days: u32,
    pub rate_shift_bps: f64,
    pub volatility_shift_pct: f64,
    pub correlation_scale: f64,
    pub equity_shocks: Vec<EquityShock>,
    pub custom_spot_shocks: Vec<SpotShock>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityShock {
    pub symbol: Symbol,
    pub expected_return_shift_pct: f64,
    pub volatility_shift_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotShock {
    pub symbol: Symbol,
    pub spot_return_shift_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloConfig {
    pub num_paths: u32,
    pub num_steps: u32,
    pub seed: u64,
    pub use_antithetic_variates: bool,
    pub use_correlated_draws: bool,
    pub sample_path_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub var_levels: Vec<f64>,
    pub compute_cvar: bool,
    pub cvar_tail_levels: Vec<f64>,
    pub compute_position_greeks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRiskResponse {
    pub request_id: RequestId,
    pub portfolio_id: PortfolioId,
    pub valuation: PortfolioValuation,
    pub aggregate_greeks: GreekVector,
    pub risk_measures: RiskMeasures,
    pub position_results: Vec<PositionRiskResult>,
    pub simulation: SimulationSummary,
    pub metadata: ResultMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioValuation {
    pub total_market_value: f64,
    pub total_pnl: f64,
    pub base_currency: CurrencyCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreekVector {
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMeasures {
    pub var_95: f64,
    pub var_99: f64,
    pub cvar_95: Option<f64>,
    pub cvar_99: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionRiskResult {
    pub position_id: PositionId,
    pub instrument: Instrument,
    pub fair_value: f64,
    pub pnl: f64,
    pub greeks: Option<GreekVector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSummary {
    pub terminal_pnl_samples: Vec<f64>,
    pub sample_paths: Vec<PathSeries>,
    pub histogram_bins: Vec<f64>,
    pub histogram_counts: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSeries {
    pub path_index: u32,
    pub prices: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultMetadata {
    pub generated_at: IsoDateTime,
    pub engine_version: String,
    pub rng_seed: u64,
    pub warnings: Vec<String>,
}
```

## Python Class Definitions

```python
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional, List

class InstrumentType(str, Enum):
    EQUITY = "equity"
    EUROPEAN_CALL_OPTION = "european_call_option"
    EUROPEAN_PUT_OPTION = "european_put_option"

class OptionStyle(str, Enum):
    EUROPEAN = "european"

class ProviderKind(str, Enum):
    ALPHA_VANTAGE = "alpha_vantage"
    FINANCIAL_MODELING_PREP = "financial_modeling_prep"
    YAHOO_FINANCE = "yahoo_finance"
    LOCAL_CACHE = "local_cache"
    HOSTED_LLM = "hosted_llm"
    LOCAL_LLM = "local_llm"

@dataclass(frozen=True)
class EquityInstrument:
    instrument_id: str
    symbol: str
    exchange: Optional[str]
    currency: str

@dataclass(frozen=True)
class EuropeanOptionInstrument:
    instrument_id: str
    underlying_symbol: str
    symbol: Optional[str]
    strike: float
    expiration_date: str
    style: OptionStyle
    multiplier: float
    currency: str
    exchange: Optional[str]

@dataclass(frozen=True)
class PortfolioPosition:
    position_id: str
    instrument: object
    quantity: float
    average_cost_basis: Optional[float] = None
    acquisition_date: Optional[str] = None

@dataclass(frozen=True)
class Portfolio:
    portfolio_id: str
    name: str
    base_currency: str
    as_of: str
    positions: List[PortfolioPosition] = field(default_factory=list)

@dataclass(frozen=True)
class MarketQuote:
    symbol: str
    spot_price: float
    currency: str
    last_updated_at: Optional[str] = None

@dataclass(frozen=True)
class ScenarioSpec:
    scenario_id: str
    name: str
    horizon_days: int
    rate_shift_bps: float
    volatility_shift_pct: float
    correlation_scale: float
    equity_shocks: list
    custom_spot_shocks: list
    notes: Optional[str] = None

@dataclass(frozen=True)
class MonteCarloConfig:
    num_paths: int
    num_steps: int
    seed: int
    use_antithetic_variates: bool = True
    use_correlated_draws: bool = True
    sample_path_count: int = 25

@dataclass(frozen=True)
class RiskConfig:
    var_levels: List[float]
    compute_cvar: bool
    cvar_tail_levels: List[float]
    compute_position_greeks: bool = True
```

## Boundary Semantics
- Python validates and enriches the payload before sending it to Rust.
- Rust performs pricing, simulation, and risk aggregation and returns only structured results.
- AI-generated scenario text is converted into `ScenarioSpec` on the Python side before reaching the engine.
- UI-facing payloads should be derived from `PortfolioRiskResponse` without additional engine-specific transformations.

## Versioning Rules
- Additive changes are allowed by appending optional fields.
- Breaking changes require a schema version bump on the top-level request and response objects.
- Field renames should be avoided; prefer deprecating old fields and adding new ones.

## Next Phase Implementation Order
1. Create the shared schema package.
2. Implement Python validators for request payloads.
3. Implement Rust serde structs with the same field names and semantics.
4. Add fixture tests that serialize in Python and deserialize in Rust, then reverse the flow.
5. Freeze the contract before wiring in the first pricing function.