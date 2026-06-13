from dataclasses import dataclass, field
from enum import Enum
from typing import Any, List, Optional


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
    instrument: Any
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
class DividendYieldPoint:
    symbol: str
    dividend_yield: float


@dataclass(frozen=True)
class VolatilityPoint:
    symbol: str
    historical_volatility: float


@dataclass(frozen=True)
class CorrelationMatrix:
    symbols: List[str]
    values: List[List[float]]


@dataclass(frozen=True)
class MarketEnvironment:
    as_of: str
    risk_free_rate: float
    dividend_yields: List[DividendYieldPoint] = field(default_factory=list)
    quotes: List[MarketQuote] = field(default_factory=list)
    volatilities: List[VolatilityPoint] = field(default_factory=list)
    correlation_matrix: Optional[CorrelationMatrix] = None


@dataclass(frozen=True)
class EquityShock:
    symbol: str
    expected_return_shift_pct: float
    volatility_shift_pct: float


@dataclass(frozen=True)
class SpotShock:
    symbol: str
    spot_return_shift_pct: float


@dataclass(frozen=True)
class ScenarioSpec:
    scenario_id: str
    name: str
    horizon_days: int
    rate_shift_bps: float
    volatility_shift_pct: float
    correlation_scale: float
    equity_shocks: List[EquityShock] = field(default_factory=list)
    custom_spot_shocks: List[SpotShock] = field(default_factory=list)
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


@dataclass(frozen=True)
class PortfolioRequest:
    request_id: str
    portfolio: Portfolio
    market: MarketEnvironment
    scenario: ScenarioSpec
    monte_carlo: MonteCarloConfig
    risk_config: RiskConfig
    compute_greeks: bool
    compute_path_samples: bool


@dataclass(frozen=True)
class PortfolioValuation:
    total_market_value: float
    total_pnl: float
    base_currency: str


@dataclass(frozen=True)
class GreekVector:
    delta: float
    gamma: float
    vega: float


@dataclass(frozen=True)
class RiskMeasures:
    var_95: float
    var_99: float
    cvar_95: Optional[float] = None
    cvar_99: Optional[float] = None


@dataclass(frozen=True)
class PositionRiskResult:
    position_id: str
    instrument: Any
    fair_value: float
    pnl: float
    greeks: Optional[GreekVector] = None


@dataclass(frozen=True)
class PathSeries:
    path_index: int
    prices: List[float]


@dataclass(frozen=True)
class SimulationSummary:
    terminal_pnl_samples: List[float] = field(default_factory=list)
    sample_paths: List[PathSeries] = field(default_factory=list)
    histogram_bins: List[float] = field(default_factory=list)
    histogram_counts: List[int] = field(default_factory=list)


@dataclass(frozen=True)
class ResultMetadata:
    generated_at: str
    engine_version: str
    rng_seed: int
    warnings: List[str] = field(default_factory=list)


@dataclass(frozen=True)
class PortfolioRiskResponse:
    request_id: str
    portfolio_id: str
    valuation: PortfolioValuation
    aggregate_greeks: GreekVector
    risk_measures: RiskMeasures
    position_results: List[PositionRiskResult]
    simulation: SimulationSummary
    metadata: ResultMetadata
