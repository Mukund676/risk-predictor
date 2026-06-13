from __future__ import annotations

from dataclasses import replace
from enum import Enum
from pathlib import Path
import sys
from typing import Any

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

ROOT = Path(__file__).resolve().parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

import risk_predictor_core  # noqa: E402
from risk_predictor.market_data import YahooFinanceProvider  # noqa: E402
from risk_predictor.report_generator import ExecutiveReportGenerator  # noqa: E402
from risk_predictor.scenario_generator import AIScenarioGenerator  # noqa: E402
from risk_predictor.schemas import (  # noqa: E402
    EquityInstrument,
    EuropeanOptionInstrument,
    MarketEnvironment,
    MonteCarloConfig,
    PathSeries,
    Portfolio,
    PortfolioPosition,
    PortfolioRequest,
    PortfolioRiskResponse,
    PortfolioValuation,
    ResultMetadata,
    RiskConfig,
    ScenarioSpec,
    SimulationSummary,
    GreekVector,
    PositionRiskResult,
    RiskMeasures,
)


app = FastAPI(title="Risk Predictor API", version="1.0.0")
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


class ScenarioRequest(BaseModel):
    prompt_text: str


def build_request(market_environment: MarketEnvironment, scenario: ScenarioSpec) -> PortfolioRequest:
    aapl = EquityInstrument(
        instrument_id="instr-aapl",
        symbol="AAPL",
        exchange="NASDAQ",
        currency="USD",
    )
    msft = EquityInstrument(
        instrument_id="instr-msft",
        symbol="MSFT",
        exchange="NASDAQ",
        currency="USD",
    )

    portfolio = Portfolio(
        portfolio_id="portfolio-001",
        name="Live Equity Portfolio",
        base_currency="USD",
        as_of=market_environment.as_of,
        positions=[
            PortfolioPosition(
                position_id="pos-aapl",
                instrument=aapl,
                quantity=12.0,
                average_cost_basis=175.00,
                acquisition_date=None,
            ),
            PortfolioPosition(
                position_id="pos-msft",
                instrument=msft,
                quantity=8.0,
                average_cost_basis=380.00,
                acquisition_date=None,
            ),
        ],
    )

    monte_carlo = MonteCarloConfig(
        num_paths=1000,
        num_steps=30,
        seed=42,
        sample_path_count=25,
    )

    risk_config = RiskConfig(
        var_levels=[0.95, 0.99],
        compute_cvar=True,
        cvar_tail_levels=[0.95, 0.99],
    )

    return PortfolioRequest(
        request_id="req-001",
        portfolio=portfolio,
        market=market_environment,
        scenario=scenario,
        monte_carlo=monte_carlo,
        risk_config=risk_config,
        compute_greeks=True,
        compute_path_samples=True,
    )


def _enum_value(value: Any) -> Any:
    if isinstance(value, Enum):
        return value.value
    return value


def _equity_instrument_to_dict(instrument: EquityInstrument) -> dict[str, Any]:
    return {
        "instrument_id": instrument.instrument_id,
        "symbol": instrument.symbol,
        "exchange": instrument.exchange,
        "currency": instrument.currency,
    }


def _option_instrument_to_dict(instrument: EuropeanOptionInstrument) -> dict[str, Any]:
    return {
        "instrument_id": instrument.instrument_id,
        "underlying_symbol": instrument.underlying_symbol,
        "symbol": instrument.symbol,
        "strike": instrument.strike,
        "expiration_date": instrument.expiration_date,
        "style": _enum_value(instrument.style),
        "multiplier": instrument.multiplier,
        "currency": instrument.currency,
        "exchange": instrument.exchange,
    }


def _instrument_to_dict(instrument: Any) -> dict[str, Any]:
    if isinstance(instrument, EquityInstrument):
        return _equity_instrument_to_dict(instrument)
    if isinstance(instrument, EuropeanOptionInstrument):
        return _option_instrument_to_dict(instrument)

    if hasattr(instrument, "underlying_symbol") and hasattr(instrument, "strike"):
        return _option_instrument_to_dict(
            EuropeanOptionInstrument(
                instrument_id=getattr(instrument, "instrument_id"),
                underlying_symbol=getattr(instrument, "underlying_symbol"),
                symbol=getattr(instrument, "symbol", None),
                strike=float(getattr(instrument, "strike")),
                expiration_date=getattr(instrument, "expiration_date"),
                style=getattr(instrument, "style"),
                multiplier=float(getattr(instrument, "multiplier", 1.0)),
                currency=getattr(instrument, "currency"),
                exchange=getattr(instrument, "exchange", None),
            )
        )

    return {
        "instrument_id": getattr(instrument, "instrument_id", None),
        "symbol": getattr(instrument, "symbol", None),
        "exchange": getattr(instrument, "exchange", None),
        "currency": getattr(instrument, "currency", None),
    }


def _portfolio_valuation_to_dict(valuation: PortfolioValuation) -> dict[str, Any]:
    return {
        "total_market_value": valuation.total_market_value,
        "total_pnl": valuation.total_pnl,
        "base_currency": valuation.base_currency,
    }


def _greek_vector_to_dict(greeks: GreekVector) -> dict[str, Any]:
    return {
        "delta": greeks.delta,
        "gamma": greeks.gamma,
        "vega": greeks.vega,
    }


def _risk_measures_to_dict(risk_measures: RiskMeasures) -> dict[str, Any]:
    return {
        "var_95": risk_measures.var_95,
        "var_99": risk_measures.var_99,
        "cvar_95": risk_measures.cvar_95,
        "cvar_99": risk_measures.cvar_99,
    }


def _position_risk_result_to_dict(position: PositionRiskResult) -> dict[str, Any]:
    return {
        "position_id": position.position_id,
        "instrument": _instrument_to_dict(position.instrument),
        "fair_value": position.fair_value,
        "pnl": position.pnl,
        "greeks": None if position.greeks is None else _greek_vector_to_dict(position.greeks),
    }


def _path_series_to_dict(path: PathSeries) -> dict[str, Any]:
    return {
        "path_index": path.path_index,
        "prices": path.prices,
    }


def _simulation_summary_to_dict(simulation: SimulationSummary) -> dict[str, Any]:
    return {
        "terminal_pnl_samples": simulation.terminal_pnl_samples,
        "sample_paths": [_path_series_to_dict(path) for path in simulation.sample_paths],
        "histogram_bins": simulation.histogram_bins,
        "histogram_counts": simulation.histogram_counts,
    }


def _metadata_to_dict(metadata: ResultMetadata) -> dict[str, Any]:
    return {
        "generated_at": metadata.generated_at,
        "engine_version": metadata.engine_version,
        "rng_seed": metadata.rng_seed,
        "warnings": metadata.warnings,
    }


def _portfolio_risk_response_to_dict(response: PortfolioRiskResponse) -> dict[str, Any]:
    return {
        "request_id": response.request_id,
        "portfolio_id": response.portfolio_id,
        "valuation": _portfolio_valuation_to_dict(response.valuation),
        "aggregate_greeks": _greek_vector_to_dict(response.aggregate_greeks),
        "risk_measures": _risk_measures_to_dict(response.risk_measures),
        "position_results": [
            _position_risk_result_to_dict(position) for position in response.position_results
        ],
        "simulation": _simulation_summary_to_dict(response.simulation),
        "metadata": _metadata_to_dict(response.metadata),
    }


@app.post("/api/simulate")
def simulate_portfolio(request: ScenarioRequest) -> dict[str, Any]:
    provider = YahooFinanceProvider(["AAPL", "MSFT"])
    market_environment = provider.fetch_market_environment()

    scenario_generator = AIScenarioGenerator()
    report_generator = ExecutiveReportGenerator()

    scenario = scenario_generator.generate_scenario(request.prompt_text)

    if market_environment.volatilities:
        baseline_vol = (
            sum(volatility.historical_volatility for volatility in market_environment.volatilities)
            / len(market_environment.volatilities)
        )
    else:
        baseline_vol = 0.0

    combined_vol = scenario.volatility_shift_pct + baseline_vol
    scenario = replace(scenario, volatility_shift_pct=combined_vol)

    portfolio_request = build_request(market_environment, scenario)
    response = risk_predictor_core.simulate_portfolio_risk(portfolio_request)
    executive_summary = report_generator.generate_summary(response, scenario.notes or "")

    return {
        "portfolio_risk_response": _portfolio_risk_response_to_dict(response),
        "executive_summary": executive_summary,
    }
