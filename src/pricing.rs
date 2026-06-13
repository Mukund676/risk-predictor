use statrs::distribution::{Continuous, ContinuousCDF, Normal};

const TIME_EPSILON: f64 = 1.0e-12;
const VOL_EPSILON: f64 = 1.0e-12;
const ONE_OVER_SQRT_2PI: f64 = 0.3989422804014327;

#[derive(Debug, Clone, Copy)]
pub struct BlackScholesInput {
    pub spot: f64,
    pub strike: f64,
    pub rate: f64,
    pub dividend_yield: f64,
    pub volatility: f64,
    pub time_to_expiry: f64,
}

impl BlackScholesInput {
    pub fn new(
        spot: f64,
        strike: f64,
        rate: f64,
        dividend_yield: f64,
        volatility: f64,
        time_to_expiry: f64,
    ) -> Self {
        Self {
            spot,
            strike,
            rate,
            dividend_yield,
            volatility,
            time_to_expiry,
        }
    }

    fn is_finite(&self) -> bool {
        self.spot.is_finite()
            && self.strike.is_finite()
            && self.rate.is_finite()
            && self.dividend_yield.is_finite()
            && self.volatility.is_finite()
            && self.time_to_expiry.is_finite()
    }

    fn valid_for_closed_form(&self) -> bool {
        self.is_finite() && self.spot > 0.0 && self.strike > 0.0 && self.volatility > 0.0 && self.time_to_expiry > 0.0
    }

    fn discount_factor(rate: f64, time: f64) -> f64 {
        (-rate * time).exp()
    }

    fn spot_discount_factor(dividend_yield: f64, time: f64) -> f64 {
        (-dividend_yield * time).exp()
    }

    fn intrinsic_call(&self) -> f64 {
        (self.spot - self.strike).max(0.0)
    }

    fn intrinsic_put(&self) -> f64 {
        (self.strike - self.spot).max(0.0)
    }

    fn deterministic_call_value(&self) -> f64 {
        let spot_discount = Self::spot_discount_factor(self.dividend_yield, self.time_to_expiry);
        let strike_discount = Self::discount_factor(self.rate, self.time_to_expiry);
        (self.spot * spot_discount - self.strike * strike_discount).max(0.0)
    }

    fn deterministic_put_value(&self) -> f64 {
        let spot_discount = Self::spot_discount_factor(self.dividend_yield, self.time_to_expiry);
        let strike_discount = Self::discount_factor(self.rate, self.time_to_expiry);
        (self.strike * strike_discount - self.spot * spot_discount).max(0.0)
    }

    fn deterministic_delta(&self, is_call: bool) -> f64 {
        let spot_discount = Self::spot_discount_factor(self.dividend_yield, self.time_to_expiry);
        let spot_discount = if spot_discount.is_finite() { spot_discount } else { 0.0 };
        let call_payoff_positive = self.deterministic_call_value() > 0.0;
        let put_payoff_positive = self.deterministic_put_value() > 0.0;

        if is_call {
            if call_payoff_positive {
                spot_discount
            } else if self.deterministic_call_value().abs() <= f64::EPSILON {
                0.5 * spot_discount
            } else {
                0.0
            }
        } else if put_payoff_positive {
            -spot_discount
        } else if self.deterministic_put_value().abs() <= f64::EPSILON {
            -0.5 * spot_discount
        } else {
            0.0
        }
    }

    fn standard_normal() -> Normal {
        Normal::new(0.0, 1.0).expect("standard normal parameters are valid")
    }

    fn d1(&self) -> f64 {
        let sigma_sqrt_t = self.volatility * self.time_to_expiry.sqrt();
        let numerator = (self.spot / self.strike).ln()
            + (self.rate - self.dividend_yield + 0.5 * self.volatility * self.volatility) * self.time_to_expiry;
        numerator / sigma_sqrt_t
    }

    fn d2(&self, d1: f64) -> f64 {
        d1 - self.volatility * self.time_to_expiry.sqrt()
    }

    fn closed_form_call_value(&self) -> f64 {
        let normal = Self::standard_normal();
        let d1 = self.d1();
        let d2 = self.d2(d1);
        let spot_discount = Self::spot_discount_factor(self.dividend_yield, self.time_to_expiry);
        let strike_discount = Self::discount_factor(self.rate, self.time_to_expiry);

        self.spot * spot_discount * normal.cdf(d1) - self.strike * strike_discount * normal.cdf(d2)
    }

    fn closed_form_put_value(&self) -> f64 {
        let normal = Self::standard_normal();
        let d1 = self.d1();
        let d2 = self.d2(d1);
        let spot_discount = Self::spot_discount_factor(self.dividend_yield, self.time_to_expiry);
        let strike_discount = Self::discount_factor(self.rate, self.time_to_expiry);

        self.strike * strike_discount * normal.cdf(-d2) - self.spot * spot_discount * normal.cdf(-d1)
    }

    fn closed_form_delta_call(&self) -> f64 {
        let normal = Self::standard_normal();
        let d1 = self.d1();
        Self::spot_discount_factor(self.dividend_yield, self.time_to_expiry) * normal.cdf(d1)
    }

    fn closed_form_delta_put(&self) -> f64 {
        self.closed_form_delta_call() - Self::spot_discount_factor(self.dividend_yield, self.time_to_expiry)
    }

    fn closed_form_gamma(&self) -> f64 {
        let normal = Self::standard_normal();
        let d1 = self.d1();
        let spot_discount = Self::spot_discount_factor(self.dividend_yield, self.time_to_expiry);
        let denominator = self.spot * self.volatility * self.time_to_expiry.sqrt();

        spot_discount * normal.pdf(d1) / denominator
    }

    fn closed_form_vega(&self) -> f64 {
        let normal = Self::standard_normal();
        let d1 = self.d1();
        let spot_discount = Self::spot_discount_factor(self.dividend_yield, self.time_to_expiry);

        self.spot * spot_discount * self.time_to_expiry.sqrt() * normal.pdf(d1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    Call,
    Put,
}

pub struct BlackScholesPricer;

impl BlackScholesPricer {
    pub fn price(input: BlackScholesInput, kind: OptionKind) -> f64 {
        if !input.is_finite() || input.spot < 0.0 || input.strike < 0.0 || input.time_to_expiry < 0.0 || input.volatility < 0.0 {
            return 0.0;
        }

        if input.strike == 0.0 {
            return match kind {
                OptionKind::Call => {
                    if input.time_to_expiry <= TIME_EPSILON {
                        input.spot
                    } else {
                        input.spot * BlackScholesInput::spot_discount_factor(input.dividend_yield, input.time_to_expiry)
                    }
                }
                OptionKind::Put => 0.0,
            };
        }

        if input.spot == 0.0 {
            return match kind {
                OptionKind::Call => 0.0,
                OptionKind::Put => {
                    if input.time_to_expiry <= TIME_EPSILON {
                        input.strike
                    } else {
                        input.strike * BlackScholesInput::discount_factor(input.rate, input.time_to_expiry)
                    }
                }
            };
        }

        if input.time_to_expiry <= TIME_EPSILON {
            return match kind {
                OptionKind::Call => input.intrinsic_call(),
                OptionKind::Put => input.intrinsic_put(),
            };
        }

        if input.volatility <= VOL_EPSILON {
            return match kind {
                OptionKind::Call => input.deterministic_call_value(),
                OptionKind::Put => input.deterministic_put_value(),
            };
        }

        if !input.valid_for_closed_form() {
            return 0.0;
        }

        let value = match kind {
            OptionKind::Call => input.closed_form_call_value(),
            OptionKind::Put => input.closed_form_put_value(),
        };

        if value.is_finite() { value.max(0.0) } else { 0.0 }
    }

    pub fn price_call(input: BlackScholesInput) -> f64 {
        Self::price(input, OptionKind::Call)
    }

    pub fn price_put(input: BlackScholesInput) -> f64 {
        Self::price(input, OptionKind::Put)
    }

    pub fn delta(input: BlackScholesInput, kind: OptionKind) -> f64 {
        if !input.is_finite() || input.spot < 0.0 || input.strike < 0.0 || input.time_to_expiry < 0.0 || input.volatility < 0.0 {
            return 0.0;
        }

        if input.strike == 0.0 {
            return match kind {
                OptionKind::Call => {
                    if input.time_to_expiry <= TIME_EPSILON {
                        1.0
                    } else {
                        BlackScholesInput::spot_discount_factor(input.dividend_yield, input.time_to_expiry)
                    }
                }
                OptionKind::Put => 0.0,
            };
        }

        if input.spot == 0.0 {
            return match kind {
                OptionKind::Call => 0.0,
                OptionKind::Put => {
                    if input.time_to_expiry <= TIME_EPSILON {
                        -1.0
                    } else {
                        -BlackScholesInput::spot_discount_factor(input.dividend_yield, input.time_to_expiry)
                    }
                }
            };
        }

        if input.time_to_expiry <= TIME_EPSILON || input.volatility <= VOL_EPSILON {
            return input.deterministic_delta(matches!(kind, OptionKind::Call));
        }

        if !input.valid_for_closed_form() {
            return 0.0;
        }

        let value = match kind {
            OptionKind::Call => input.closed_form_delta_call(),
            OptionKind::Put => input.closed_form_delta_put(),
        };

        if value.is_finite() { value } else { 0.0 }
    }

    pub fn delta_call(input: BlackScholesInput) -> f64 {
        Self::delta(input, OptionKind::Call)
    }

    pub fn delta_put(input: BlackScholesInput) -> f64 {
        Self::delta(input, OptionKind::Put)
    }

    pub fn gamma(input: BlackScholesInput) -> f64 {
        if !input.is_finite() || input.spot <= 0.0 || input.strike <= 0.0 || input.time_to_expiry <= TIME_EPSILON || input.volatility <= VOL_EPSILON {
            return 0.0;
        }

        let value = input.closed_form_gamma();
        if value.is_finite() { value.max(0.0) } else { 0.0 }
    }

    pub fn vega(input: BlackScholesInput) -> f64 {
        if !input.is_finite() || input.spot <= 0.0 || input.strike <= 0.0 || input.time_to_expiry <= TIME_EPSILON || input.volatility <= VOL_EPSILON {
            return 0.0;
        }

        let value = input.closed_form_vega();
        if value.is_finite() { value.max(0.0) } else { 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::{BlackScholesInput, BlackScholesPricer, OptionKind};

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        let difference = (actual - expected).abs();
        assert!(
            difference <= tolerance,
            "expected {expected}, got {actual}, difference {difference} exceeded tolerance {tolerance}"
        );
    }

    #[test]
    fn prices_match_reference_values() {
        let input = BlackScholesInput::new(100.0, 100.0, 0.05, 0.0, 0.2, 1.0);

        assert_close(BlackScholesPricer::price_call(input), 10.450583572185565, 1e-12);
        assert_close(BlackScholesPricer::price_put(input), 5.573526022256971, 1e-12);
    }

    #[test]
    fn greeks_match_reference_values() {
        let input = BlackScholesInput::new(100.0, 100.0, 0.05, 0.0, 0.2, 1.0);

        assert_close(BlackScholesPricer::delta_call(input), 0.6368306511756191, 1e-12);
        assert_close(BlackScholesPricer::delta_put(input), -0.3631693488243809, 1e-12);
        assert_close(BlackScholesPricer::gamma(input), 0.018762017345846895, 1e-12);
        assert_close(BlackScholesPricer::vega(input), 37.52403469169379, 1e-12);
    }

    #[test]
    fn put_call_parity_holds_for_closed_form_path() {
        let input = BlackScholesInput::new(120.0, 100.0, 0.03, 0.01, 0.25, 2.0);
        let call = BlackScholesPricer::price_call(input);
        let put = BlackScholesPricer::price_put(input);
        let parity_rhs = input.spot * (-input.dividend_yield * input.time_to_expiry).exp()
            - input.strike * (-input.rate * input.time_to_expiry).exp();

        assert_close(call - put, parity_rhs, 1e-10);
    }

    #[test]
    fn zero_time_returns_intrinsic_value() {
        let call_input = BlackScholesInput::new(130.0, 100.0, 0.07, 0.01, 0.35, 0.0);
        let put_input = BlackScholesInput::new(80.0, 100.0, 0.07, 0.01, 0.35, 0.0);

        assert_eq!(BlackScholesPricer::price(call_input, OptionKind::Call), 30.0);
        assert_eq!(BlackScholesPricer::price(put_input, OptionKind::Put), 20.0);
        assert_eq!(BlackScholesPricer::gamma(call_input), 0.0);
        assert_eq!(BlackScholesPricer::vega(call_input), 0.0);
    }

    #[test]
    fn zero_volatility_returns_deterministic_value_without_nan() {
        let call_input = BlackScholesInput::new(100.0, 95.0, 0.05, 0.02, 0.0, 1.0);
        let put_input = BlackScholesInput::new(90.0, 100.0, 0.05, 0.02, 0.0, 1.0);

        let call = BlackScholesPricer::price_call(call_input);
        let put = BlackScholesPricer::price_put(put_input);

        assert!(call.is_finite());
        assert!(put.is_finite());
        assert_close(call, (100.0 * (-0.02f64).exp() - 95.0 * (-0.05f64).exp()).max(0.0), 1e-12);
        assert_close(put, (100.0 * (-0.05f64).exp() - 90.0 * (-0.02f64).exp()).max(0.0), 1e-12);
    }

    #[test]
    fn extreme_inputs_do_not_produce_nan() {
        let input = BlackScholesInput::new(0.0, 100.0, 0.05, 0.0, 0.2, 1.0);

        let call = BlackScholesPricer::price_call(input);
        let put = BlackScholesPricer::price_put(input);
        let delta_call = BlackScholesPricer::delta_call(input);
        let delta_put = BlackScholesPricer::delta_put(input);
        let gamma = BlackScholesPricer::gamma(input);
        let vega = BlackScholesPricer::vega(input);

        assert!(call.is_finite());
        assert!(put.is_finite());
        assert!(delta_call.is_finite());
        assert!(delta_put.is_finite());
        assert!(gamma.is_finite());
        assert!(vega.is_finite());
        assert_eq!(call, 0.0);
        assert_close(put, 100.0 * (-0.05f64).exp(), 1e-12);
    }
}
