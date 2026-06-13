from dataclasses import replace
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / "python"))

import risk_predictor_core  # noqa: E402
from risk_predictor.market_data import YahooFinanceProvider  # noqa: E402
from risk_predictor.report_generator import ExecutiveReportGenerator  # noqa: E402
from risk_predictor.scenario_generator import AIScenarioGenerator  # noqa: E402
from risk_predictor.schemas import (  # noqa: E402
    EquityInstrument,
    MonteCarloConfig,
    Portfolio,
    PortfolioPosition,
    PortfolioRequest,
    RiskConfig,
)


def build_request(market_environment, scenario) -> PortfolioRequest:
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
                average_cost_basis=175.00, # Added Cost Basis
                acquisition_date=None,
            ),
            PortfolioPosition(
                position_id="pos-msft",
                instrument=msft,
                quantity=8.0,
                average_cost_basis=380.00, # Added Cost Basis
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


def main() -> None:
    provider = YahooFinanceProvider(["AAPL", "MSFT"])
    market_environment = provider.fetch_market_environment()
    scenario_generator = AIScenarioGenerator()
    report_generator = ExecutiveReportGenerator()

    while True:
        prompt_text = input("Describe your stress scenario (or type 'exit'): ").strip()
        if prompt_text.lower() == "exit":
            break
        if not prompt_text:
            print("Please enter a non-empty stress scenario description.")
            continue

        scenario = scenario_generator.generate_scenario(prompt_text)

        if market_environment.volatilities:
            baseline_vol = (
                sum(volatility.historical_volatility for volatility in market_environment.volatilities)
                / len(market_environment.volatilities)
            )
        else:
            baseline_vol = 0.0

        combined_vol = scenario.volatility_shift_pct + baseline_vol

        scenario = replace(scenario, volatility_shift_pct=combined_vol)

        request = build_request(market_environment, scenario)
        response = risk_predictor_core.simulate_portfolio_risk(request)
        executive_summary = report_generator.generate_summary(response, scenario.notes or "")

        print("\n=== PORTFOLIO RISK REPORT ===")
        print(f"Request ID:     {response.request_id}")
        print(f"Portfolio ID:   {response.portfolio_id}")
        print(f"Engine Version:  {response.metadata.engine_version}")
        print(f"As Of:           {response.metadata.generated_at}")

        print("\n--- SCENARIO ---")
        print(f"Name:           {scenario.name}")
        print(f"Horizon Days:   {scenario.horizon_days}")
        print(f"Rate Shift BPS: {scenario.rate_shift_bps:,.2f}")
        print(f"Vol Shift Pct:  {scenario.volatility_shift_pct:,.2%}")
        print(f"Corr Scale:     {scenario.correlation_scale:,.2f}")
        if scenario.notes:
            print(f"Notes:          {scenario.notes}")

        print("\n--- VALUATION ---")
        print(f"Total Market Value: ${response.valuation.total_market_value:,.2f}")
        print(f"Total PnL:          ${response.valuation.total_pnl:,.2f}")

        print("\n--- AGGREGATE GREEKS ---")
        print(f"Delta: {response.aggregate_greeks.delta:,.4f}")
        print(f"Gamma: {response.aggregate_greeks.gamma:,.4f}")
        print(f"Vega:  {response.aggregate_greeks.vega:,.4f}")

        print("\n--- TAIL RISK ---")
        print(f"95% VaR:  ${response.risk_measures.var_95:,.2f}")
        print(f"95% CVaR: ${response.risk_measures.cvar_95 or 0.0:,.2f}")
        print(f"99% VaR:  ${response.risk_measures.var_99:,.2f}")
        print(f"99% CVaR: ${response.risk_measures.cvar_99 or 0.0:,.2f}")

        print("\n--- POSITION BREAKDOWN ---")
        for position in response.position_results:
            print(f"Position: {position.position_id}")
            print(f"  Fair Value: ${position.fair_value:,.2f}")
            print(f"  PnL:        ${position.pnl:,.2f}")
            if position.greeks:
                print(f"  Delta:      {position.greeks.delta:,.4f}")
                print(f"  Gamma:      {position.greeks.gamma:,.4f}")
                print(f"  Vega:       {position.greeks.vega:,.4f}")

        print("\n--- SIMULATION SUMMARY ---")
        print(f"Terminal samples: {len(response.simulation.terminal_pnl_samples)}")
        print(f"Sample paths:     {len(response.simulation.sample_paths)}")
        print(f"Histogram bins:   {len(response.simulation.histogram_bins)}")
        print("=============================\n")

        print("\n=== EXECUTIVE SUMMARY ===")
        print(executive_summary)


if __name__ == "__main__":
    main()