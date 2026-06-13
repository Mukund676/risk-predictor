import { useMemo, useState } from 'react'
import {
  AlertTriangle,
  ArrowRight,
  BarChart3,
  Brain,
  LoaderCircle,
  LineChart as LineChartIcon,
  Radar,
  Sparkles,
  TrendingUp,
} from 'lucide-react'
import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'

const API_URL = 'http://127.0.0.1:8000/api/simulate'
const USD = new Intl.NumberFormat('en-US', {
  style: 'currency',
  currency: 'USD',
  maximumFractionDigits: 2,
})

const PATH_COLORS = ['#22d3ee', '#34d399', '#f59e0b', '#f472b6', '#a78bfa', '#60a5fa', '#fb7185', '#facc15']

function formatCurrency(value) {
  return USD.format(Number.isFinite(value) ? value : 0)
}

function buildMonteCarloChartData(samplePaths) {
  const paths = Array.isArray(samplePaths) ? samplePaths : []
  const maxSteps = paths.reduce((max, path) => Math.max(max, path?.prices?.length ?? 0), 0)

  return Array.from({ length: maxSteps }, (_, step) => {
    const row = { step }
    paths.forEach((path, index) => {
      row[`path_${index}`] = path?.prices?.[step] ?? null
    })
    return row
  })
}

function buildHistogramData(histogramBins, histogramCounts) {
  const bins = Array.isArray(histogramBins) ? histogramBins : []
  const counts = Array.isArray(histogramCounts) ? histogramCounts : []

  return bins.map((bin, index) => ({
    bin,
    count: counts[index] ?? 0,
  }))
}

function App() {
  const [scenarioPrompt, setScenarioPrompt] = useState(
    'A sharp equity selloff with higher rates, wider credit spreads, and a volatility spike.',
  )
  const [isLoading, setIsLoading] = useState(false)
  const [reportData, setReportData] = useState(null)

  const portfolioResponse = reportData?.portfolio_risk_response ?? null
  const valuation = portfolioResponse?.valuation ?? {}
  const riskMeasures = portfolioResponse?.risk_measures ?? {}
  const simulation = portfolioResponse?.simulation ?? {}

  const monteCarloData = useMemo(
    () => buildMonteCarloChartData(simulation.sample_paths),
    [simulation.sample_paths],
  )

  const histogramData = useMemo(
    () => buildHistogramData(simulation.histogram_bins, simulation.histogram_counts),
    [simulation.histogram_bins, simulation.histogram_counts],
  )

  const handleSubmit = async (event) => {
    event.preventDefault()
    if (!scenarioPrompt.trim()) return

    setIsLoading(true)
    try {
      const response = await fetch(API_URL, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ prompt_text: scenarioPrompt }),
      })

      if (!response.ok) {
        throw new Error(`Request failed with status ${response.status}`)
      }

      const payload = await response.json()
      setReportData(payload)
    } catch (error) {
      console.error('Simulation request failed:', error)
      setReportData(null)
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100">
      <div className="absolute inset-0 -z-10 bg-[radial-gradient(circle_at_top_left,_rgba(34,211,238,0.16),_transparent_30%),radial-gradient(circle_at_top_right,_rgba(16,185,129,0.12),_transparent_28%),linear-gradient(180deg,_rgba(15,23,42,0.96),_rgba(2,6,23,1))]" />

      <main className="mx-auto flex min-h-screen w-full max-w-[1600px] flex-col gap-6 px-4 py-4 sm:px-6 lg:px-8">
        <header className="rounded-3xl border border-slate-800/80 bg-slate-900/70 p-4 shadow-2xl shadow-cyan-950/20 backdrop-blur">
          <div className="flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between">
            <div className="max-w-3xl space-y-2">
              <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.3em] text-cyan-300">
                <Radar className="h-4 w-4" />
                Investment Banking Risk Predictor
              </div>
              <h1 className="text-3xl font-semibold tracking-tight text-white sm:text-4xl">
                Executive Risk Dashboard
              </h1>
              <p className="max-w-2xl text-sm leading-6 text-slate-400 sm:text-base">
                Convert a stress scenario into a live Monte Carlo risk view, then review the tail
                distribution, key VaR metrics, and an AI-generated executive summary.
              </p>
            </div>

            <form onSubmit={handleSubmit} className="flex w-full flex-col gap-3 xl:max-w-4xl xl:flex-row xl:items-center">
              <div className="relative flex-1">
                <Brain className="pointer-events-none absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-cyan-400" />
                <input
                  value={scenarioPrompt}
                  onChange={(event) => setScenarioPrompt(event.target.value)}
                  placeholder="Describe the market shock, e.g. 'credit spread widening, rates up 75 bps, tech selloff'"
                  className="w-full rounded-2xl border border-slate-700 bg-slate-950/80 py-4 pl-12 pr-4 text-sm text-slate-100 outline-none transition placeholder:text-slate-500 focus:border-cyan-400 focus:ring-2 focus:ring-cyan-400/20"
                />
              </div>

              <button
                type="submit"
                disabled={isLoading}
                className="inline-flex items-center justify-center gap-2 rounded-2xl bg-cyan-500 px-5 py-4 text-sm font-semibold text-slate-950 transition hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-70"
              >
                {isLoading ? (
                  <>
                    <LoaderCircle className="h-4 w-4 animate-spin" />
                    Running
                  </>
                ) : (
                  <>
                    Run Simulation
                    <ArrowRight className="h-4 w-4" />
                  </>
                )}
              </button>
            </form>
          </div>
        </header>

        {reportData ? (
          <>
            <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
              <MetricCard title="Total Market Value" value={formatCurrency(valuation.total_market_value)} accent="cyan" icon={TrendingUp} />
              <MetricCard title="Total PnL" value={formatCurrency(valuation.total_pnl)} accent="emerald" icon={Sparkles} />
              <MetricCard title="95% VaR" value={formatCurrency(riskMeasures.var_95)} accent="amber" icon={AlertTriangle} />
              <MetricCard title="99% VaR" value={formatCurrency(riskMeasures.var_99)} accent="rose" icon={BarChart3} />
            </section>

            <section className="grid gap-6 xl:grid-cols-[1.2fr_0.8fr]">
              <div className="rounded-3xl border border-slate-800 bg-slate-900/75 p-5 shadow-xl shadow-cyan-950/10 backdrop-blur">
                <div className="mb-4 flex items-center justify-between gap-4">
                  <div>
                    <div className="text-xs font-semibold uppercase tracking-[0.25em] text-cyan-300">Executive Summary</div>
                    <h2 className="mt-1 text-lg font-semibold text-white">Chief Risk Officer View</h2>
                  </div>
                  <LineChartIcon className="h-5 w-5 text-cyan-400" />
                </div>
                <div className="prose prose-invert max-w-none rounded-2xl border border-slate-800 bg-slate-950/70 p-5 text-sm leading-7 text-slate-300 prose-p:my-0">
                  <p className="whitespace-pre-line">{reportData.executive_summary}</p>
                </div>
              </div>

              <div className="rounded-3xl border border-slate-800 bg-slate-900/75 p-5 shadow-xl shadow-cyan-950/10 backdrop-blur">
                <div className="mb-4 flex items-center justify-between gap-4">
                  <div>
                    <div className="text-xs font-semibold uppercase tracking-[0.25em] text-cyan-300">Scenario Notes</div>
                    <h2 className="mt-1 text-lg font-semibold text-white">Input Context</h2>
                  </div>
                  <Sparkles className="h-5 w-5 text-emerald-400" />
                </div>
                <div className="rounded-2xl border border-slate-800 bg-slate-950/70 p-5 text-sm leading-7 text-slate-300">
                  {portfolioResponse?.metadata?.warnings?.length ? (
                    <div className="mb-4 rounded-xl border border-amber-500/30 bg-amber-500/10 p-3 text-amber-200">
                      {portfolioResponse.metadata.warnings.join(' ')}
                    </div>
                  ) : null}
                  <p className="whitespace-pre-line text-slate-300">
                    {portfolioResponse?.scenario?.notes ?? 'Scenario notes were embedded into the AI summary request.'}
                  </p>
                </div>
              </div>
            </section>

            <section className="grid gap-6 xl:grid-cols-2">
              <ChartPanel
                title="Monte Carlo Paths"
                subtitle="Each line is a simulated terminal price path indexed by step"
                icon={LineChartIcon}
              >
                <ResponsiveContainer width="100%" height={420}>
                  <LineChart data={monteCarloData} margin={{ top: 10, right: 20, left: 0, bottom: 0 }}>
                    <CartesianGrid strokeDasharray="3 3" stroke="rgba(148,163,184,0.12)" />
                    <XAxis dataKey="step" stroke="#94a3b8" tickLine={false} axisLine={false} />
                    <YAxis stroke="#94a3b8" tickLine={false} axisLine={false} tickFormatter={(value) => formatCurrency(value)} width={90} />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: '#020617',
                        border: '1px solid rgba(148,163,184,0.25)',
                        borderRadius: '16px',
                        color: '#e2e8f0',
                      }}
                      labelStyle={{ color: '#67e8f9' }}
                      formatter={(value) => formatCurrency(value)}
                    />
                    <Legend />
                    {simulation.sample_paths?.map((path, index) => (
                      <Line
                        key={path.path_index ?? index}
                        type="monotone"
                        dataKey={`path_${index}`}
                        name={`Path ${index + 1}`}
                        stroke={PATH_COLORS[index % PATH_COLORS.length]}
                        strokeWidth={1.75}
                        dot={false}
                        activeDot={false}
                        isAnimationActive={false}
                        strokeOpacity={0.9}
                      />
                    ))}
                  </LineChart>
                </ResponsiveContainer>
              </ChartPanel>

              <ChartPanel
                title="Tail Risk Distribution"
                subtitle="Histogram of terminal PnL outcomes"
                icon={BarChart3}
              >
                <ResponsiveContainer width="100%" height={420}>
                  <BarChart data={histogramData} margin={{ top: 10, right: 20, left: 0, bottom: 0 }}>
                    <CartesianGrid strokeDasharray="3 3" stroke="rgba(148,163,184,0.12)" />
                    <XAxis
                      dataKey="bin"
                      stroke="#94a3b8"
                      tickLine={false}
                      axisLine={false}
                      tickFormatter={(value) => formatCurrency(value)}
                      minTickGap={18}
                    />
                    <YAxis stroke="#94a3b8" tickLine={false} axisLine={false} />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: '#020617',
                        border: '1px solid rgba(148,163,184,0.25)',
                        borderRadius: '16px',
                        color: '#e2e8f0',
                      }}
                      labelFormatter={(value) => `PnL Bin: ${formatCurrency(value)}`}
                    />
                    <Bar dataKey="count" radius={[8, 8, 0, 0]}>
                      {histogramData.map((entry, index) => (
                        <Cell
                          key={`cell-${entry.bin}-${index}`}
                          fill={index % 2 === 0 ? '#22d3ee' : '#0ea5e9'}
                          fillOpacity={0.85}
                        />
                      ))}
                    </Bar>
                  </BarChart>
                </ResponsiveContainer>
              </ChartPanel>
            </section>
          </>
        ) : (
          <section className="grid flex-1 place-items-center rounded-3xl border border-slate-800 bg-slate-900/55 px-6 py-20 text-center shadow-2xl shadow-cyan-950/10 backdrop-blur">
            <div className="max-w-2xl space-y-4">
              <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-full border border-cyan-400/30 bg-cyan-500/10 text-cyan-300">
                <Sparkles className="h-7 w-7" />
              </div>
              <h2 className="text-2xl font-semibold text-white">Ready for a stress scenario</h2>
              <p className="text-sm leading-7 text-slate-400">
                Enter a market shock narrative above to generate the scenario, run the Rust Monte
                Carlo engine, and surface a live dashboard with risk metrics and executive commentary.
              </p>
            </div>
          </section>
        )}
      </main>
    </div>
  )
}

function MetricCard({ title, value, accent, icon: Icon }) {
  const accentClasses = {
    cyan: 'from-cyan-500/20 to-cyan-500/5 border-cyan-400/20 text-cyan-300',
    emerald: 'from-emerald-500/20 to-emerald-500/5 border-emerald-400/20 text-emerald-300',
    amber: 'from-amber-500/20 to-amber-500/5 border-amber-400/20 text-amber-300',
    rose: 'from-rose-500/20 to-rose-500/5 border-rose-400/20 text-rose-300',
  }

  return (
    <article className={`rounded-3xl border bg-gradient-to-br p-5 shadow-xl shadow-black/20 ${accentClasses[accent]}`}>
      <div className="flex items-start justify-between gap-4">
        <div>
          <div className="text-xs font-semibold uppercase tracking-[0.28em] text-slate-400">{title}</div>
          <div className="mt-3 text-3xl font-semibold tracking-tight text-white">{value}</div>
        </div>
        <div className="rounded-2xl border border-white/10 bg-white/5 p-3 text-white/80">
          <Icon className="h-5 w-5" />
        </div>
      </div>
    </article>
  )
}

function ChartPanel({ title, subtitle, icon: Icon, children }) {
  return (
    <section className="rounded-3xl border border-slate-800 bg-slate-900/75 p-5 shadow-xl shadow-cyan-950/10 backdrop-blur">
      <div className="mb-4 flex items-start justify-between gap-4">
        <div>
          <div className="text-xs font-semibold uppercase tracking-[0.25em] text-cyan-300">{title}</div>
          <h3 className="mt-1 text-lg font-semibold text-white">{subtitle}</h3>
        </div>
        <Icon className="h-5 w-5 text-cyan-400" />
      </div>
      <div className="rounded-2xl border border-slate-800 bg-slate-950/70 p-3">{children}</div>
    </section>
  )
}

export default App
