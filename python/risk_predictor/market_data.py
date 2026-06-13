from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from datetime import datetime, timezone
import math
import statistics
from typing import Dict, Iterable, List, Sequence

from .schemas import (
    DividendYieldPoint,
    MarketEnvironment,
    MarketQuote,
    VolatilityPoint,
)

try:
    import yfinance as yf
except ImportError as error:  # pragma: no cover - import error is environment-specific
    raise ImportError(
        "yfinance is required to fetch live market data. Install it with `pip install yfinance`."
    ) from error


def _utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def _clean_numeric_values(values: Iterable[float]) -> List[float]:
    cleaned: List[float] = []
    for value in values:
        try:
            numeric_value = float(value)
        except (TypeError, ValueError):
            continue
        if math.isfinite(numeric_value):
            cleaned.append(numeric_value)
    return cleaned


@dataclass(frozen=True)
class _TickerSnapshot:
    symbol: str
    spot_price: float
    dividend_yield: float
    historical_volatility: float


class YahooFinanceProvider:
    def __init__(self, tickers: Sequence[str]):
        self._tickers = [ticker.strip().upper() for ticker in tickers if ticker and ticker.strip()]
        if not self._tickers:
            raise ValueError("at least one ticker symbol is required")

    def fetch_market_environment(self) -> MarketEnvironment:
        snapshots = self._fetch_snapshots()

        quotes = [
            MarketQuote(
                symbol=snapshot.symbol,
                spot_price=snapshot.spot_price,
                currency="USD",
                last_updated_at=_utc_now_iso(),
            )
            for snapshot in snapshots
        ]

        dividend_yields = [
            DividendYieldPoint(symbol=snapshot.symbol, dividend_yield=snapshot.dividend_yield)
            for snapshot in snapshots
        ]

        volatilities = [
            VolatilityPoint(symbol=snapshot.symbol, historical_volatility=snapshot.historical_volatility)
            for snapshot in snapshots
        ]

        return MarketEnvironment(
            as_of=_utc_now_iso(),
            risk_free_rate=0.0425,
            dividend_yields=dividend_yields,
            quotes=quotes,
            volatilities=volatilities,
            correlation_matrix=None,
        )

    def _fetch_snapshots(self) -> List[_TickerSnapshot]:
        max_workers = min(len(self._tickers), 8)
        with ThreadPoolExecutor(max_workers=max_workers) as executor:
            future_map = {
                executor.submit(self._fetch_single_ticker, symbol): symbol for symbol in self._tickers
            }
            results: Dict[str, _TickerSnapshot] = {}
            for future in as_completed(future_map):
                snapshot = future.result()
                results[snapshot.symbol] = snapshot

        return [results[symbol] for symbol in self._tickers]

    def _fetch_single_ticker(self, symbol: str) -> _TickerSnapshot:
        ticker = yf.Ticker(symbol)
        history = ticker.history(period="2y", interval="1d", auto_adjust=False, actions=True)

        spot_price = self._extract_spot_price(ticker, history)
        dividend_yield = self._extract_dividend_yield(ticker, history, spot_price)
        historical_volatility = self._extract_historical_volatility(history)

        return _TickerSnapshot(
            symbol=symbol,
            spot_price=spot_price,
            dividend_yield=dividend_yield,
            historical_volatility=historical_volatility,
        )

    def _extract_spot_price(self, ticker: yf.Ticker, history) -> float:
        fast_info = getattr(ticker, "fast_info", {}) or {}
        for key in ("lastPrice", "last_price", "regularMarketPrice"):
            value = fast_info.get(key)
            if value is not None and math.isfinite(float(value)) and float(value) > 0.0:
                return float(value)

        close_values = _clean_numeric_values(history["Close"].dropna().tolist()) if not history.empty and "Close" in history else []
        if close_values:
            return close_values[-1]

        info = getattr(ticker, "info", {}) or {}
        for key in ("currentPrice", "regularMarketPrice", "previousClose"):
            value = info.get(key)
            if value is not None and math.isfinite(float(value)) and float(value) > 0.0:
                return float(value)

        raise ValueError(f"unable to determine spot price for {ticker.ticker}")

    def _extract_dividend_yield(self, ticker: yf.Ticker, history, spot_price: float) -> float:
        info = getattr(ticker, "info", {}) or {}
        for key in ("trailingAnnualDividendYield", "dividendYield"):
            value = info.get(key)
            if value is not None:
                numeric_value = float(value)
                if math.isfinite(numeric_value) and numeric_value >= 0.0:
                    return numeric_value

        trailing_dividends = 0.0
        dividends = getattr(ticker, "dividends", None)
        if dividends is not None and len(dividends) > 0:
            try:
                trailing_dividends = float(dividends.last("365D").sum())
            except Exception:
                trailing_dividends = float(dividends.tail(10).sum())

        if spot_price > 0.0 and math.isfinite(spot_price):
            yield_value = trailing_dividends / spot_price
            if math.isfinite(yield_value) and yield_value >= 0.0:
                return yield_value

        return 0.0

    def _extract_historical_volatility(self, history) -> float:
        if history.empty or "Close" not in history:
            return 0.0

        close_prices = _clean_numeric_values(history["Close"].dropna().tolist())
        if len(close_prices) < 2:
            return 0.0

        closing_window = close_prices[-253:]
        if len(closing_window) < 2:
            return 0.0

        log_returns: List[float] = []
        for previous_close, current_close in zip(closing_window, closing_window[1:]):
            if previous_close > 0.0 and current_close > 0.0:
                log_returns.append(math.log(current_close / previous_close))

        if len(log_returns) < 2:
            return 0.0

        return statistics.stdev(log_returns) * math.sqrt(252.0)
