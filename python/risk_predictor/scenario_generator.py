from __future__ import annotations

import os

from google import genai
from google.genai import types
from pydantic import BaseModel, Field

from .schemas import ScenarioSpec


class ScenarioExtraction(BaseModel):
    horizon_days: int = Field(default=30, ge=1)
    rate_shift_bps: float
    volatility_shift_pct: float
    correlation_scale: float = Field(default=1.0)
    notes: str


class AIScenarioGenerator:
    def __init__(self, model: str = "gemini-2.5-flash") -> None:
        self._model = model

    def generate_scenario(self, prompt_text: str) -> ScenarioSpec:
        api_key = os.getenv("GEMINI_API_KEY")
        if not api_key:
            raise ValueError("GEMINI_API_KEY is not set")

        client = genai.Client(api_key=api_key)
        response = client.models.generate_content(
            model=self._model,
            contents=prompt_text,
            config=types.GenerateContentConfig(
                response_mime_type="application/json",
                response_schema=ScenarioExtraction,
            ),
        )

        extraction = response.parsed
        if extraction is None:
            raise ValueError("Gemini did not return a structured scenario payload")

        if not isinstance(extraction, ScenarioExtraction):
            extraction = ScenarioExtraction.model_validate(extraction)

        # --- THE FIX: Normalize the volatility decimal ---
        raw_vol_shift = extraction.volatility_shift_pct
        normalized_vol_shift = raw_vol_shift / 100.0 if abs(raw_vol_shift) > 1.0 else raw_vol_shift
        

        return ScenarioSpec(
            scenario_id="scenario-ai-generated",
            name="AI-generated stress scenario",
            horizon_days=extraction.horizon_days,
            rate_shift_bps=extraction.rate_shift_bps,
            volatility_shift_pct=normalized_vol_shift, # Pass the normalized value here!
            correlation_scale=extraction.correlation_scale,
            equity_shocks=[],
            custom_spot_shocks=[],
            notes=extraction.notes,
        )