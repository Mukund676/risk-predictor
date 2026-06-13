from __future__ import annotations

import os

from google import genai
from google.genai import types

from .schemas import PortfolioRiskResponse


class ExecutiveReportGenerator:
    def __init__(self, model: str = "gemini-3.5-flash") -> None:
        api_key = os.getenv("GEMINI_API_KEY")
        if not api_key:
            raise ValueError("GEMINI_API_KEY is not set")

        self._client = genai.Client(api_key=api_key)
        self._model = model

    def generate_summary(self, response: PortfolioRiskResponse, scenario_notes: str) -> str:
        data_block = self._build_data_block(response, scenario_notes)
        system_prompt = (
            "You are a Chief Risk Officer analyzing a portfolio stress test. "
            "Based on the following quantitative data, write a highly professional, "
            "two-paragraph executive summary explaining the severity of the tail risk, "
            "the impact of the scenario, and what it means for the portfolio."
        )

        # --- THE FIX: Try/Except block for Graceful Degradation ---
        try:
            generation = self._client.models.generate_content(
                model=self._model,
                contents=data_block,
                config=types.GenerateContentConfig(
                    system_instruction=system_prompt,
                ),
            )

            summary = (generation.text or "").strip()
            if not summary:
                raise ValueError("Gemini did not return an executive summary")

            return summary
            
        except Exception as e:
            # If the API throws a 503 or 429, catch it and return a fallback message
            print(f"Warning: AI Summary generation failed: {e}")
            return (
                "Executive Summary is temporarily unavailable due to high API demand. "
                "The quantitative risk calculations have successfully completed and are available in the data payload."
            )
    def _build_data_block(self, response: PortfolioRiskResponse, scenario_notes: str) -> str:
        notes = scenario_notes.strip() or "None provided"
        return (
            "Portfolio Stress Test Data\n"
            f"Request ID: {response.request_id}\n"
            f"Portfolio ID: {response.portfolio_id}\n"
            f"Total Market Value: ${response.valuation.total_market_value:,.2f}\n"
            f"PnL: ${response.valuation.total_pnl:,.2f}\n"
            f"95% VaR: ${response.risk_measures.var_95:,.2f}\n"
            f"99% VaR: ${response.risk_measures.var_99:,.2f}\n"
            f"Scenario Notes: {notes}\n"
        )
