# Formal Delivery Plan: Investment Banking Risk & Pricing Predictor

## Delivery Assumptions
- Build cadence: evening and weekend development on a standard 40-hour work week.
- Target window: early June through mid-August 2026.
- Delivery intent: a working MVP with a contract-first boundary, deterministic pricing engine, Monte Carlo risk simulation, AI scenario/report generation, and a dashboard-ready API shape.
- Scope guardrails: European equities and options only, Black-Scholes pricing, GBM Monte Carlo, VaR/CVaR, AI as narrator and scenario interpreter only.

## Milestone Map

### Sprint 1: Jun 8 - Jun 14
**Goal:** Establish the system boundary and repository skeleton.

**Deliverables**
- Canonical schema draft for portfolio, instrument, scenario, and simulation contracts.
- Repository layout for Python orchestration, Rust core, shared schema definitions, UI, and docs.
- Serialization decision for the first cut of the Python/Rust boundary.
- Test strategy for contract validation and deterministic replay.

**Dependencies**
- None. This is the foundation sprint.

### Sprint 2: Jun 15 - Jun 21
**Goal:** Lock the data contract and validation layer.

**Deliverables**
- Finalized schema contract for request/response payloads.
- Validation rules for instrument identifiers, dates, quantities, rates, volatilities, and scenario inputs.
- Fixture set for sample portfolios and market states.
- Initial Python model classes and Rust struct definitions for shared payloads.

**Dependencies**
- Sprint 1 schema draft and repository skeleton.

### Sprint 3: Jun 22 - Jun 28
**Goal:** Implement deterministic pricing and Greeks.

**Deliverables**
- Black-Scholes pricing for European calls and puts.
- Delta, gamma, and vega for each option contract.
- Numerical edge-case handling for zero time-to-expiry, near-zero volatility, and deep ITM/OTM cases.
- Reference-value tests against known analytical outputs.

**Dependencies**
- Sprint 2 final schema and validation rules.

### Sprint 4: Jun 29 - Jul 5
**Goal:** Build the Monte Carlo core.

**Deliverables**
- Geometric Brownian Motion path generator.
- Seedable random number stream and deterministic reruns.
- Batched path simulation and CPU parallelization.
- Sampling controls for scenario analysis and UI-friendly path subsets.

**Dependencies**
- Sprint 2 data contract.
- Sprint 3 market pricing primitives for path valuation.

### Sprint 5: Jul 6 - Jul 12
**Goal:** Compute portfolio-level risk metrics.

**Deliverables**
- Aggregated P&L distribution from simulated paths.
- VaR at 95% and 99% confidence levels.
- CVaR / expected shortfall at 95% and 99% confidence levels.
- Greeks aggregation at portfolio and position level.
- Statistical sanity checks for convergence and stability.

**Dependencies**
- Sprint 4 Monte Carlo engine.

### Sprint 6: Jul 13 - Jul 19
**Goal:** Add market data ingestion and normalization.

**Deliverables**
- Provider adapters for Alpha Vantage, Financial Modeling Prep, and Yahoo Finance or equivalent sources.
- Normalization pipeline for prices, implied/historical volatility, and rates.
- Caching, batching, retry, and backoff behavior for API resilience.
- Data freshness and provenance metadata.

**Dependencies**
- Sprint 2 canonical market data schema.
- Risk engine contracts from Sprints 3-5.

### Sprint 7: Jul 20 - Jul 26
**Goal:** Add AI scenario translation.

**Deliverables**
- AI provider abstraction supporting hosted and local models.
- Prompt-to-scenario translation into explicit numerical shocks.
- Scenario parameter mapping for rates, volatility, correlations, and sector shocks.
- Guardrails to prevent the AI from altering pricing math.

**Dependencies**
- Sprint 2 scenario schema.
- Sprint 5 risk output shape.

### Sprint 8: Jul 27 - Aug 2
**Goal:** Add report synthesis and auditability.

**Deliverables**
- Executive risk report generator using simulation outputs.
- Hedging recommendation scaffolding based on portfolio vulnerabilities.
- Scenario provenance and audit log design.
- Reproducible narrative generation contract for the AI layer.

**Dependencies**
- Sprint 7 scenario engine.
- Sprint 5 portfolio risk metrics.

### Sprint 9: Aug 3 - Aug 9
**Goal:** Build the API and frontend shell.

**Deliverables**
- Portfolio submission and simulation endpoints.
- Result retrieval endpoints for portfolio risk, path samples, and scenario summaries.
- React dashboard with table input, Greeks panel, line chart, and histogram panels.
- Mocked data flow from API to chart components.

**Dependencies**
- Sprint 5 result payloads.
- Sprint 8 report output shape.

### Sprint 10: Aug 10 - Aug 16
**Goal:** Harden the MVP and prepare for handoff.

**Deliverables**
- End-to-end integration tests.
- Performance profiling of the Rust engine.
- Deterministic replay tests for pricing, simulation, and scenario generation.
- Documentation for architecture, data contract, and runtime assumptions.
- Packaging and deployment notes for local development.

**Dependencies**
- All prior sprints.

## Dependencies
- The canonical schema must stabilize before any serious engine or API work begins.
- Pricing and Greeks must be validated before Monte Carlo risk metrics are trusted.
- Monte Carlo and risk aggregation must stabilize before AI summaries are generated, because the AI should summarize finished outputs only.
- Data ingestion should plug into the same contract used by the core engine; no alternate market model should be introduced.
- Frontend implementation should wait until response payloads are stable enough to avoid expensive rework.

## Implementation Backlog

### P0 - Must Build First
- Define shared data contracts for portfolios, instruments, market state, scenarios, and risk outputs.
- Establish Rust serde structs and Python Pydantic models with identical field semantics.
- Add schema validation and fixture-based contract tests.
- Implement Black-Scholes pricing for European options.
- Implement delta, gamma, and vega.

### P1 - Core Risk Engine
- Build GBM Monte Carlo simulation with deterministic seeds.
- Parallelize path generation across CPU cores.
- Compute VaR and CVaR from simulated P&L distributions.
- Add portfolio aggregation over mixed equity and option holdings.
- Add reference tests and benchmark harnesses.

### P2 - Data and Orchestration
- Add market data adapters and normalization.
- Create caching and retry policies for remote feeds.
- Implement request orchestration in Python.
- Wire portfolio input into the Rust engine and return structured risk responses.

### P3 - AI Layer
- Implement provider abstraction for hosted and local models.
- Build prompt-to-scenario translation.
- Build executive report synthesis from risk outputs.
- Add AI prompt/version logging for auditability.

### P4 - UI and Delivery
- Build the React dashboard shell.
- Add portfolio table entry, Greeks visualization, Monte Carlo path chart, and tail-risk histogram.
- Connect frontend to the API and mocked data.
- Add end-to-end integration and smoke tests.

## Delivery Criteria
- The system can ingest a sample portfolio, price options, run 10,000+ Monte Carlo paths, and report VaR/CVaR consistently.
- The AI layer can convert a natural-language shock prompt into a structured scenario and generate a concise executive summary.
- The UI can render portfolio inputs and risk outputs from the same canonical payloads used by the engine.
- Results are reproducible under fixed seeds and pass basic reference checks for pricing and Greeks.