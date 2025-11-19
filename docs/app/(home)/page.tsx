import Link from 'next/link';
import { ArrowRight, BarChart3, ShieldCheck, Zap, Layers, Cpu, Globe } from 'lucide-react';

export default function HomePage() {
  return (
    <main className="flex flex-col min-h-screen bg-white dark:bg-black transition-colors duration-300">
      {/* Hero Section */}
      <section className="relative py-24 md:py-32 overflow-hidden border-b border-zinc-200 dark:border-zinc-800">
        <div className="container mx-auto px-4 text-center z-10 relative">
          {/* Badge */}
          <div className="inline-flex items-center gap-2 rounded-full border border-zinc-200 bg-zinc-50/80 px-3 py-1 text-sm text-zinc-600 mb-8 backdrop-blur-sm dark:border-zinc-800 dark:bg-zinc-900/50 dark:text-zinc-400">
            <span className="flex h-2 w-2 rounded-full bg-green-500 animate-pulse"></span>
            v0.2.3 Stable Release
          </div>
          
          {/* Headline */}
          <h1 className="text-5xl md:text-7xl font-bold tracking-tight text-zinc-900 mb-6 dark:text-transparent dark:bg-clip-text dark:bg-gradient-to-b dark:from-white dark:to-white/60">
            Institutional-Grade <br />
            <span className="text-blue-600 dark:text-blue-500">Quantitative Trading</span>
          </h1>
          
          {/* Subheadline */}
          <p className="text-lg md:text-xl text-zinc-600 max-w-2xl mx-auto mb-10 leading-relaxed dark:text-zinc-400">
            Tesser is a Rust-native framework designed for high-frequency reliability. 
            We decouple execution logic from market connectivity, enabling 
            <strong className="text-zinc-900 dark:text-zinc-200 font-semibold"> zero-cost abstractions</strong> and 
            <strong className="text-zinc-900 dark:text-zinc-200 font-semibold"> robust risk management</strong>.
          </p>
          
          {/* CTA Buttons */}
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link
              href="/docs"
              className="flex items-center gap-2 rounded-full bg-blue-600 px-8 py-3 text-sm font-semibold text-white hover:bg-blue-700 transition-all shadow-lg shadow-blue-600/20 dark:bg-blue-600 dark:hover:bg-blue-500 dark:shadow-blue-900/20"
            >
              Get Started <ArrowRight className="size-4" />
            </Link>
            <Link
              href="https://github.com/tesserspace/tesser"
              target="_blank"
              className="flex items-center gap-2 rounded-full border border-zinc-200 bg-white px-8 py-3 text-sm font-semibold text-zinc-700 hover:bg-zinc-50 hover:text-zinc-900 transition-all dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800"
            >
              View Source
            </Link>
          </div>
        </div>
        
        {/* Light Mode Background Grid (Subtle) */}
        <div className="absolute inset-0 bg-[linear-gradient(to_right,#80808012_1px,transparent_1px),linear-gradient(to_bottom,#80808012_1px,transparent_1px)] bg-[size:24px_24px] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_0%,#000_70%,transparent_100%)] pointer-events-none dark:opacity-0" />

        {/* Dark Mode Gradient Effect (Blue Glow) */}
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[1000px] h-[400px] bg-blue-500/10 blur-[100px] rounded-full pointer-events-none -z-10 opacity-0 dark:opacity-100" />
      </section>

      {/* Value Proposition Grid */}
      <section className="py-24 bg-zinc-50 dark:bg-zinc-950/30">
        <div className="container mx-auto px-4">
          <div className="text-center mb-16">
            <h2 className="text-3xl font-bold text-zinc-900 mb-4 dark:text-white">Why Tesser?</h2>
            <p className="text-zinc-600 dark:text-zinc-400 max-w-2xl mx-auto">
              Built for traders who demand the performance of low-level systems with the safety of modern software engineering.
            </p>
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <FeatureCard 
              icon={<Zap className="size-6 text-amber-500" />}
              title="Rust Performance"
              description="Leverage Rust's zero-cost abstractions. Handle high-frequency ticks and order book deltas with minimal latency and no garbage collection pauses."
            />
            <FeatureCard 
              icon={<ShieldCheck className="size-6 text-emerald-500" />}
              title="Safety & Reconciliation"
              description="Built-in state persistence and real-time reconciliation prevent portfolio drift. The system automatically enters 'liquidate-only' mode if discrepancies are detected."
            />
            <FeatureCard 
              icon={<Layers className="size-6 text-purple-500" />}
              title="Modular Architecture"
              description="Strict decoupling between strategies and exchanges. Switch from Paper Trading to Bybit Live execution without changing a single line of strategy code."
            />
            <FeatureCard 
              icon={<BarChart3 className="size-6 text-blue-500" />}
              title="Accurate Backtesting"
              description="Event-driven engine supports both candle and tick-level simulations (L2/L3 data), accurately modeling latency, slippage, and maker/taker fees."
            />
            <FeatureCard 
              icon={<Cpu className="size-6 text-rose-500" />}
              title="WASM Plugins (Roadmap)"
              description="Future-proof design allows dynamic loading of strategies via WebAssembly, enabling hot-reloading and multi-language strategy development."
            />
            <FeatureCard 
              icon={<Globe className="size-6 text-cyan-500" />}
              title="Unified API"
              description="A normalized trait system handles the complexity of REST and WebSocket APIs across exchanges, providing a consistent interface for your algorithms."
            />
          </div>
        </div>
      </section>

      {/* Technical Preview */}
      <section className="py-24 bg-white dark:bg-black border-t border-zinc-200 dark:border-zinc-800">
        <div className="container mx-auto px-4">
          <div className="flex flex-col lg:flex-row items-center gap-16">
            <div className="flex-1 space-y-8">
              <h3 className="text-3xl font-bold text-zinc-900 dark:text-white">Developer Experience First</h3>
              <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed text-lg">
                We believe powerful tools shouldn't be painful to use. Tesser provides a declarative TOML configuration system, a robust CLI for operations, and a type-safe SDK for strategy authoring.
              </p>
              <ul className="space-y-4">
                <li className="flex items-center gap-3">
                  <div className="flex items-center justify-center w-6 h-6 rounded-full bg-blue-100 dark:bg-blue-900/30">
                    <div className="h-2 w-2 rounded-full bg-blue-600 dark:bg-blue-500" />
                  </div>
                  <span className="text-zinc-700 dark:text-zinc-300">Type-safe <code className="text-xs bg-zinc-100 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 px-1.5 py-0.5 rounded text-zinc-900 dark:text-zinc-200 font-mono">rust_decimal</code> for financial precision</span>
                </li>
                <li className="flex items-center gap-3">
                  <div className="flex items-center justify-center w-6 h-6 rounded-full bg-blue-100 dark:bg-blue-900/30">
                    <div className="h-2 w-2 rounded-full bg-blue-600 dark:bg-blue-500" />
                  </div>
                  <span className="text-zinc-700 dark:text-zinc-300">Built-in technical indicators (RSI, MACD, Bollinger)</span>
                </li>
                <li className="flex items-center gap-3">
                  <div className="flex items-center justify-center w-6 h-6 rounded-full bg-blue-100 dark:bg-blue-900/30">
                    <div className="h-2 w-2 rounded-full bg-blue-600 dark:bg-blue-500" />
                  </div>
                  <span className="text-zinc-700 dark:text-zinc-300">Multi-process architecture for isolation</span>
                </li>
              </ul>
            </div>
            
            {/* Code Window - Keep dark even in light mode for contrast/IDE feel */}
            <div className="flex-1 w-full max-w-lg rounded-xl overflow-hidden shadow-2xl border border-zinc-200 dark:border-zinc-800 bg-[#0d1117] dark:bg-zinc-900">
              <div className="flex items-center gap-2 px-4 py-3 border-b border-zinc-800 bg-zinc-900/50">
                <div className="flex gap-1.5">
                  <div className="w-3 h-3 rounded-full bg-red-500/20 border border-red-500/50" />
                  <div className="w-3 h-3 rounded-full bg-yellow-500/20 border border-yellow-500/50" />
                  <div className="w-3 h-3 rounded-full bg-green-500/20 border border-green-500/50" />
                </div>
                <span className="text-xs text-zinc-500 font-mono ml-2">strategies/sma_cross.toml</span>
              </div>
              <div className="p-6 overflow-x-auto">
                <pre className="text-sm font-mono text-zinc-300 leading-relaxed">
                  <code>{`strategy_name = "SmaCross"

[params]
symbol = "BTCUSDT"
fast_period = 8
slow_period = 21
min_samples = 48

# Risk Guardrails
vwap_duration_secs = 600
vwap_participation = 0.2`}</code>
                </pre>
              </div>
            </div>
          </div>
        </div>
      </section>
    </main>
  );
}

function FeatureCard({ icon, title, description }: { icon: React.ReactNode, title: string, description: string }) {
  return (
    <div className="group p-6 rounded-2xl border border-zinc-200 bg-white shadow-sm hover:shadow-md hover:border-zinc-300 transition-all dark:border-zinc-800 dark:bg-zinc-900/20 dark:hover:bg-zinc-900/40 dark:hover:border-zinc-700 dark:shadow-none">
      <div className="mb-4 p-3 rounded-lg bg-zinc-50 border border-zinc-100 inline-block group-hover:bg-white transition-colors dark:bg-zinc-900 dark:border-zinc-800 dark:group-hover:border-zinc-700">
        {icon}
      </div>
      <h3 className="text-xl font-semibold mb-3 text-zinc-900 dark:text-zinc-100">{title}</h3>
      <p className="text-zinc-600 text-sm leading-relaxed dark:text-zinc-400">{description}</p>
    </div>
  );
}
