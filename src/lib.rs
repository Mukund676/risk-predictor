mod monte_carlo;
mod pricing;
mod pytypes;

use monte_carlo::{simulate_portfolio_monte_carlo, GbmRequest, MonteCarloConfig as MonteCarloEngineConfig, MonteCarloResult, PathSeries as MonteCarloPathSeries};
use pricing::{BlackScholesInput, BlackScholesPricer, OptionKind};
use pyo3::prelude::*;
use pytypes::{
    CorrelationMatrix, DividendYieldPoint, EquityInstrument, EquityShock, EuropeanOptionInstrument,
    GreekVector, MarketEnvironment, MarketQuote, MonteCarloConfig, PathSeries, Portfolio,
    PortfolioPosition, PortfolioRequest, PortfolioRiskResponse, PortfolioValuation,
    PositionRiskResult, ResultMetadata, RiskConfig, RiskMeasures, ScenarioSpec, SimulationSummary,
    SpotShock, VolatilityPoint,
};

const DEFAULT_OPTION_MULTIPLIER: f64 = 100.0;
const DEFAULT_WEIGHT: f64 = 1.0;
const DAYS_IN_YEAR: f64 = 365.0;

fn get_string_attr(object: &Bound<'_, PyAny>, name: &str) -> PyResult<String> {
    object.getattr(name)?.extract::<String>()
}

fn get_f64_attr(object: &Bound<'_, PyAny>, name: &str) -> PyResult<f64> {
    object.getattr(name)?.extract::<f64>()
}

fn get_optional_f64_attr(object: &Bound<'_, PyAny>, name: &str) -> PyResult<Option<f64>> {
    match object.getattr(name) {
        Ok(value) if value.is_none() => Ok(None),
        Ok(value) => Ok(Some(value.extract::<f64>()?)),
        Err(_) => Ok(None),
    }
}

fn parse_date_components(date: &str) -> PyResult<(i32, u32, u32)> {
    if date.len() < 10 {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "invalid date '{date}', expected YYYY-MM-DD"
        )));
    }

    let date_part = &date[..10];
    let mut parts = date_part.split('-');

    let year = parts
        .next()
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing year"))?
        .parse::<i32>()
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(format!("invalid year: {error}")))?;
    let month = parts
        .next()
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing month"))?
        .parse::<u32>()
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(format!("invalid month: {error}")))?;
    let day = parts
        .next()
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing day"))?
        .parse::<u32>()
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(format!("invalid day: {error}")))?;

    Ok((year, month, day))
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let month = month as i32;
    let day = day as i32;
    let adjusted_year = year - if month <= 2 { 1 } else { 0 };
    let era = if adjusted_year >= 0 { adjusted_year } else { adjusted_year - 399 } / 400;
    let year_of_era = adjusted_year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    (era as i64) * 146_097 + day_of_era as i64 - 719_468
}

fn time_to_expiry_years(as_of: &str, expiration_date: &str) -> PyResult<f64> {
    let (as_of_year, as_of_month, as_of_day) = parse_date_components(as_of)?;
    let (expiry_year, expiry_month, expiry_day) = parse_date_components(expiration_date)?;

    let days = days_from_civil(expiry_year, expiry_month, expiry_day)
        - days_from_civil(as_of_year, as_of_month, as_of_day);
    Ok((days.max(0) as f64) / DAYS_IN_YEAR)
}

fn find_symbol_value(list_object: &Bound<'_, PyAny>, symbol: &str, value_attr: &str) -> PyResult<f64> {
    for entry in list_object.iter()? {
        let item = entry?;
        let item_symbol = get_string_attr(&item, "symbol")?;
        if item_symbol == symbol {
            return get_f64_attr(&item, value_attr);
        }
    }

    Err(pyo3::exceptions::PyValueError::new_err(format!(
        "missing {value_attr} for symbol '{symbol}'"
    )))
}

fn infer_option_kind(instrument: &Bound<'_, PyAny>) -> OptionKind {
    if let Ok(option_type_value) = instrument.getattr("instrument_type") {
        if let Ok(value) = option_type_value.extract::<String>() {
            let lower = value.to_ascii_lowercase();
            if lower.contains("put") {
                return OptionKind::Put;
            }
            if lower.contains("call") {
                return OptionKind::Call;
            }
        }
    }

    OptionKind::Call
}

fn extract_option_context(
    instrument: &Bound<'_, PyAny>,
    market: &Bound<'_, PyAny>,
    portfolio_as_of: &str,
) -> PyResult<(OptionKind, BlackScholesInput, f64)> {
    let underlying_symbol = get_string_attr(instrument, "underlying_symbol")?;
    let strike = get_f64_attr(instrument, "strike")?;
    let expiration_date = get_string_attr(instrument, "expiration_date")?;
    let multiplier = get_optional_f64_attr(instrument, "multiplier")?.unwrap_or(DEFAULT_OPTION_MULTIPLIER);
    let multiplier = if multiplier.is_finite() && multiplier > 0.0 {
        multiplier
    } else {
        DEFAULT_OPTION_MULTIPLIER
    };

    let spot_price = find_symbol_value(&market.getattr("quotes")?, &underlying_symbol, "spot_price")?;
    let volatility = find_symbol_value(&market.getattr("volatilities")?, &underlying_symbol, "historical_volatility")?;
    let risk_free_rate = get_f64_attr(market, "risk_free_rate")?;
    let dividend_yield = match market.getattr("dividend_yields") {
        Ok(dividend_yields) => find_symbol_value(&dividend_yields, &underlying_symbol, "dividend_yield").unwrap_or(0.0),
        Err(_) => 0.0,
    };
    let time_to_expiry = time_to_expiry_years(portfolio_as_of, &expiration_date)?;
    let option_kind = infer_option_kind(instrument);

    let pricing_input = BlackScholesInput::new(
        spot_price,
        strike,
        risk_free_rate,
        dividend_yield,
        volatility,
        time_to_expiry,
    );

    Ok((option_kind, pricing_input, multiplier))
}

fn to_mc_config(config: &MonteCarloConfig) -> MonteCarloEngineConfig {
    MonteCarloEngineConfig::new(
        config.num_paths as usize,
        config.num_steps as usize,
        config.seed,
        config.sample_path_count as usize,
    )
}

fn scenario_weighted_drift_and_volatility(scenario: &Bound<'_, PyAny>) -> PyResult<(f64, f64)> {
    let base_drift = get_f64_attr(scenario, "rate_shift_bps").unwrap_or(0.0) / 10_000.0;
    let base_vol_shift = get_f64_attr(scenario, "volatility_shift_pct").unwrap_or(0.0);

    let mut drift_sum = 0.0_f64;
    let mut vol_sum = 0.0_f64;
    let mut weight_sum = 0.0_f64;

    if let Ok(equity_shocks) = scenario.getattr("equity_shocks") {
        for entry in equity_shocks.iter()? {
            let shock = entry?;
            let expected_return_shift_pct = get_f64_attr(&shock, "expected_return_shift_pct")?;
            let volatility_shift_pct = get_f64_attr(&shock, "volatility_shift_pct")?;
            let weight = DEFAULT_WEIGHT;
            drift_sum += expected_return_shift_pct * weight;
            vol_sum += volatility_shift_pct * weight;
            weight_sum += weight;
        }
    }

    let blended_drift = if weight_sum > 0.0 {
        drift_sum / weight_sum
    } else {
        0.0
    } + base_drift;

    let blended_volatility = if weight_sum > 0.0 {
        vol_sum / weight_sum
    } else {
        0.0
    } + base_vol_shift;

    Ok((blended_drift, blended_volatility.abs()))
}

fn build_histogram(samples: &[f64], bin_count: usize) -> (Vec<f64>, Vec<u32>) {
    if samples.is_empty() || bin_count == 0 {
        return (Vec::new(), Vec::new());
    }

    let min_value = samples.iter().copied().fold(f64::INFINITY, f64::min);
    let max_value = samples.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    if !min_value.is_finite() || !max_value.is_finite() {
        return (Vec::new(), Vec::new());
    }

    if (max_value - min_value).abs() <= f64::EPSILON {
        return (vec![min_value], vec![samples.len() as u32]);
    }

    let width = (max_value - min_value) / bin_count as f64;
    let mut bins = Vec::with_capacity(bin_count);
    let mut counts = vec![0_u32; bin_count];

    for index in 0..bin_count {
        bins.push(min_value + width * index as f64);
    }

    for &sample in samples {
        let mut bin_index = ((sample - min_value) / width).floor() as isize;
        if bin_index < 0 {
            bin_index = 0;
        }
        if bin_index as usize >= bin_count {
            bin_index = bin_count as isize - 1;
        }
        counts[bin_index as usize] += 1;
    }

    (bins, counts)
}

fn calculate_var_and_cvar(sorted_samples: &[f64], confidence: f64) -> (f64, f64) {
    if sorted_samples.is_empty() {
        return (0.0, 0.0);
    }

    let clamped_confidence = confidence.clamp(0.0, 1.0);
    let tail_fraction = 1.0 - clamped_confidence;
    let mut var_index = (tail_fraction * sorted_samples.len() as f64).ceil() as usize;
    if var_index == 0 {
        var_index = 1;
    }
    var_index = var_index.min(sorted_samples.len());

    let var_value = sorted_samples[var_index - 1];
    let cvar_slice = &sorted_samples[..var_index];
    let cvar_value = if cvar_slice.is_empty() {
        var_value
    } else {
        cvar_slice.iter().copied().sum::<f64>() / cvar_slice.len() as f64
    };

    (var_value, cvar_value)
}

fn monte_carlo_result_to_summary(py: Python<'_>, result: MonteCarloResult) -> PyResult<SimulationSummary> {
    let sample_paths = result
        .sample_paths
        .into_iter()
        .map(|path| Py::new(
            py,
            PathSeries {
                path_index: path.path_index as u32,
                prices: path.prices,
            },
        ))
        .collect::<PyResult<Vec<_>>>()?;

    let (histogram_bins, histogram_counts) = build_histogram(&result.terminal_pnl_samples, 12);

    Ok(SimulationSummary {
        terminal_pnl_samples: result.terminal_pnl_samples,
        sample_paths,
        histogram_bins,
        histogram_counts,
    })
}

fn price_option_position(
    py: Python<'_>,
    position: &Bound<'_, PyAny>,
    instrument: &Bound<'_, PyAny>,
    market: &Bound<'_, PyAny>,
    portfolio_as_of: &str,
    quantity: f64,
) -> PyResult<(f64, f64, Py<GreekVector>)> {
    let (option_kind, pricing_input, multiplier) = extract_option_context(instrument, market, portfolio_as_of)?;

    let per_unit_price = match option_kind {
        OptionKind::Call => BlackScholesPricer::price_call(pricing_input),
        OptionKind::Put => BlackScholesPricer::price_put(pricing_input),
    };
    let per_unit_delta = match option_kind {
        OptionKind::Call => BlackScholesPricer::delta_call(pricing_input),
        OptionKind::Put => BlackScholesPricer::delta_put(pricing_input),
    };
    let per_unit_gamma = BlackScholesPricer::gamma(pricing_input);
    let per_unit_vega = BlackScholesPricer::vega(pricing_input);

    let fair_value = per_unit_price * quantity * multiplier;
    let delta = per_unit_delta * quantity * multiplier;
    let gamma = per_unit_gamma * quantity * multiplier;
    let vega = per_unit_vega * quantity * multiplier;

    let average_cost_basis = get_optional_f64_attr(position, "average_cost_basis")?.unwrap_or(0.0);
    let pnl = fair_value - average_cost_basis * quantity * multiplier;

    let greeks = Py::new(
        py,
        GreekVector {
            delta,
            gamma,
            vega,
        },
    )?;

    Ok((fair_value, pnl, greeks))
}

fn price_equity_position(
    py: Python<'_>,
    position: &Bound<'_, PyAny>,
    instrument: &Bound<'_, PyAny>,
    market: &Bound<'_, PyAny>,
    quantity: f64,
) -> PyResult<(f64, f64, Py<GreekVector>)> {
    let symbol = get_string_attr(instrument, "symbol")?;
    let spot_price = find_symbol_value(&market.getattr("quotes")?, &symbol, "spot_price")?;
    let fair_value = spot_price * quantity;
    let average_cost_basis = get_optional_f64_attr(position, "average_cost_basis")?.unwrap_or(0.0);
    let pnl = fair_value - average_cost_basis * quantity;

    let greeks = Py::new(
        py,
        GreekVector {
            delta: quantity,
            gamma: 0.0,
            vega: 0.0,
        },
    )?;

    Ok((fair_value, pnl, greeks))
}

#[pyfunction]
fn simulate_portfolio_risk(py: Python<'_>, request: &Bound<'_, PyAny>) -> PyResult<Py<PortfolioRiskResponse>> {
    let request_id = get_string_attr(request, "request_id")?;
    let portfolio = request.getattr("portfolio")?;
    let market = request.getattr("market")?;
    let scenario = request.getattr("scenario")?;
    let monte_carlo_config = request.getattr("monte_carlo")?;
    let risk_config = request.getattr("risk_config")?;

    let portfolio_id = get_string_attr(&portfolio, "portfolio_id")?;
    let portfolio_as_of = get_string_attr(&portfolio, "as_of")?;
    let base_currency = get_string_attr(&portfolio, "base_currency")?;
    let positions = portfolio.getattr("positions")?;

    let (blended_drift, blended_volatility_shift) = scenario_weighted_drift_and_volatility(&scenario)?;
    let market_rate = get_f64_attr(&market, "risk_free_rate")?;
    let drift = market_rate + blended_drift;
    let volatility = blended_volatility_shift.max(0.0);

    let mut total_market_value = 0.0_f64;
    let mut total_pnl = 0.0_f64;
    let mut aggregate_delta = 0.0_f64;
    let mut aggregate_gamma = 0.0_f64;
    let mut aggregate_vega = 0.0_f64;
    let mut position_results: Vec<Py<PositionRiskResult>> = Vec::new();

    for position_entry in positions.iter()? {
        let position = position_entry?;
        let position_id = get_string_attr(&position, "position_id")?;
        let quantity = get_f64_attr(&position, "quantity")?;
        let instrument = position.getattr("instrument")?;
        let instrument_object = instrument.clone().unbind();

        let class_name = instrument.get_type().name()?.to_string();

        let (fair_value, pnl, greeks) = if class_name == "EuropeanOptionInstrument" {
            price_option_position(py, &position, &instrument, &market, &portfolio_as_of, quantity)?
        } else if class_name == "EquityInstrument" {
            price_equity_position(py, &position, &instrument, &market, quantity)?
        } else {
            return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                "unsupported instrument type '{class_name}' for position '{position_id}'"
            )));
        };

        let greeks_value = greeks.bind(py).borrow();
        aggregate_delta += greeks_value.delta;
        aggregate_gamma += greeks_value.gamma;
        aggregate_vega += greeks_value.vega;

        total_market_value += fair_value;
        total_pnl += pnl;

        let position_result = Py::new(
            py,
            PositionRiskResult {
                position_id,
                instrument: instrument_object,
                fair_value,
                pnl,
                greeks: Some(greeks),
            },
        )?;
        position_results.push(position_result);
    }

    let valuation = Py::new(
        py,
        PortfolioValuation {
            total_market_value,
            total_pnl,
            base_currency,
        },
    )?;

    let aggregate_greeks = Py::new(
        py,
        GreekVector {
            delta: aggregate_delta,
            gamma: aggregate_gamma,
            vega: aggregate_vega,
        },
    )?;

    let mc_config = MonteCarloEngineConfig::new(
        get_f64_attr(&monte_carlo_config, "num_paths")?.max(1.0) as usize,
        get_f64_attr(&monte_carlo_config, "num_steps")?.max(1.0) as usize,
        get_f64_attr(&monte_carlo_config, "seed")? as u64,
        get_f64_attr(&monte_carlo_config, "sample_path_count")?.max(1.0) as usize,
    );
    let horizon_years = get_f64_attr(&scenario, "horizon_days")? / DAYS_IN_YEAR;
    let gbm_request = GbmRequest::new(
        total_market_value,
        drift,
        volatility,
        horizon_years,
        total_market_value,
        mc_config,
    );

    let monte_carlo_result = simulate_portfolio_monte_carlo(&gbm_request);
    let mut sorted_terminal_pnl_samples = monte_carlo_result.terminal_pnl_samples.clone();
    sorted_terminal_pnl_samples.sort_by(|left, right| left.total_cmp(right));

    let var_levels = risk_config.getattr("var_levels")?;
    let mut var_95 = 0.0;
    let mut var_99 = 0.0;
    let mut cvar_95 = None;
    let mut cvar_99 = None;

    for level_entry in var_levels.iter()? {
        let level = level_entry?.extract::<f64>()?;
        if (level - 0.95).abs() <= 1.0e-12 {
            let (var, cvar) = calculate_var_and_cvar(&sorted_terminal_pnl_samples, level);
            var_95 = var;
            cvar_95 = Some(cvar);
        } else if (level - 0.99).abs() <= 1.0e-12 {
            let (var, cvar) = calculate_var_and_cvar(&sorted_terminal_pnl_samples, level);
            var_99 = var;
            cvar_99 = Some(cvar);
        }
    }

    let (histogram_bins, histogram_counts) = build_histogram(&sorted_terminal_pnl_samples, 12);
    let mut simulation_summary = monte_carlo_result_to_summary(py, monte_carlo_result)?;
    simulation_summary.terminal_pnl_samples = sorted_terminal_pnl_samples;
    simulation_summary.histogram_bins = histogram_bins;
    simulation_summary.histogram_counts = histogram_counts;

    let simulation = Py::new(py, simulation_summary)?;

    let risk_measures = Py::new(
        py,
        RiskMeasures {
            var_95,
            var_99,
            cvar_95,
            cvar_99,
        },
    )?;

    let metadata = Py::new(
        py,
        ResultMetadata {
            generated_at: portfolio_as_of,
            engine_version: "sprint-4-monte-carlo-core".to_string(),
            rng_seed: 0,
            warnings: Vec::new(),
        },
    )?;

    Py::new(
        py,
        PortfolioRiskResponse {
            request_id,
            portfolio_id,
            valuation,
            aggregate_greeks,
            risk_measures,
            position_results,
            simulation,
            metadata,
        },
    )
}

#[pymodule]
fn risk_predictor_core(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<EquityInstrument>()?;
    m.add_class::<EuropeanOptionInstrument>()?;
    m.add_class::<PortfolioPosition>()?;
    m.add_class::<Portfolio>()?;
    m.add_class::<MarketQuote>()?;
    m.add_class::<DividendYieldPoint>()?;
    m.add_class::<VolatilityPoint>()?;
    m.add_class::<CorrelationMatrix>()?;
    m.add_class::<MarketEnvironment>()?;
    m.add_class::<EquityShock>()?;
    m.add_class::<SpotShock>()?;
    m.add_class::<ScenarioSpec>()?;
    m.add_class::<MonteCarloConfig>()?;
    m.add_class::<RiskConfig>()?;
    m.add_class::<PortfolioRequest>()?;
    m.add_class::<PortfolioValuation>()?;
    m.add_class::<GreekVector>()?;
    m.add_class::<RiskMeasures>()?;
    m.add_class::<PositionRiskResult>()?;
    m.add_class::<PathSeries>()?;
    m.add_class::<SimulationSummary>()?;
    m.add_class::<ResultMetadata>()?;
    m.add_class::<PortfolioRiskResponse>()?;
    m.add_function(wrap_pyfunction!(simulate_portfolio_risk, m)?)?;
    Ok(())
}
