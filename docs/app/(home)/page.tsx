'use client';

import Link from 'next/link';
import { useEffect, useState, useRef } from 'react';
import { 
  ArrowRight, BarChart3, ShieldCheck, Zap, Layers, Cpu, Globe, 
  Terminal, Activity, Lock, ChevronRight, Github 
} from 'lucide-react';
import { motion, useScroll, useTransform, useInView } from 'framer-motion';
import { cn } from '@/lib/utils';

// --- Components ---

function HeroSection() {
  return (
    <section className="relative pt-32 pb-20 md:pt-48 md:pb-32 overflow-hidden">
      <div className="container mx-auto px-4 relative z-10">
        <div className="flex flex-col items-center text-center">
          <motion.div 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
            className="inline-flex items-center gap-2 rounded-full border border-blue-500/20 bg-blue-500/10 px-3 py-1 text-sm text-blue-600 dark:text-blue-400 mb-8 backdrop-blur-md"
          >
            <span className="relative flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-blue-400 opacity-75"></span>
              <span className="relative inline-flex rounded-full h-2 w-2 bg-blue-500"></span>
            </span>
            v0.2.3 Stable Release
          </motion.div>
          
          <motion.h1 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-5xl md:text-8xl font-bold tracking-tight text-zinc-900 dark:text-white mb-6"
          >
            Quantitative Trading <br />
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-blue-600 to-cyan-500 dark:from-blue-400 dark:to-cyan-300">
              Reimagined in Rust
            </span>
          </motion.h1>
          
          <motion.p 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            className="text-xl text-zinc-600 dark:text-zinc-400 max-w-2xl mx-auto mb-10 leading-relaxed"
          >
            The institutional-grade framework for high-frequency reliability.
            Decoupled architecture, zero-cost abstractions, and robust risk guards.
          </motion.p>
          
          <motion.div 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            className="flex flex-col sm:flex-row items-center gap-4"
          >
            <Link
              href="/docs"
              className="flex items-center gap-2 rounded-full bg-zinc-900 dark:bg-white px-8 py-4 text-base font-semibold text-white dark:text-black hover:opacity-90 transition-all shadow-xl shadow-blue-500/10"
            >
              Start Building <ArrowRight className="size-4" />
            </Link>
            <Link
              href="https://github.com/tesserspace/tesser"
              target="_blank"
              className="flex items-center gap-2 rounded-full border border-zinc-200 dark:border-zinc-800 bg-white/50 dark:bg-zinc-900/50 px-8 py-4 text-base font-semibold text-zinc-700 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-all backdrop-blur-sm"
            >
              <Github className="size-4" /> GitHub
            </Link>
          </motion.div>
        </div>
      </div>

      {/* Background Effects */}
      <div className="absolute top-0 inset-x-0 h-[600px] bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-blue-500/10 via-transparent to-transparent opacity-50 pointer-events-none dark:from-blue-500/20" />
      <div className="absolute inset-0 bg-[url('/grid.svg')] bg-center [mask-image:linear-gradient(180deg,white,rgba(255,255,255,0))]" />
    </section>
  );
}

function TerminalSimulation() {
  const [lines, setLines] = useState<string[]>([
    "Initializing Tesser v0.2.3...",
    "Loading strategy configuration...",
    "Connecting to Bybit Linear (WebSocket)...",
  ]);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const sequence = [
      { text: "✓ Market stream connected: BTCUSDT", delay: 800 },
      { text: "✓ Execution engine ready (Live Mode)", delay: 1600 },
      { text: "INFO [Strategy] SmaCross initialized (fast=8, slow=21)", delay: 2400 },
      { text: "INFO [Risk] Pre-trade checks active (max_drawdown=5%)", delay: 3000 },
      { text: "WARN [Market] High volatility detected, adjusting spreads", delay: 4500 },
      { text: "EXEC [Order] SUBMIT LIMIT BUY 1.5 BTC @ 64200.50", delay: 6000 },
      { text: "EXEC [Fill] FILLED 1.5 BTC @ 64200.50 (12ms latency)", delay: 7500 },
      { text: "INFO [Portfolio] Position updated: +1.5 BTC (PnL: +0.05%)", delay: 8200 },
    ];

    let timeouts: NodeJS.Timeout[] = [];

    sequence.forEach(({ text, delay }) => {
      const timeout = setTimeout(() => {
        setLines(prev => [...prev.slice(-7), text]); // Keep last 8 lines
      }, delay);
      timeouts.push(timeout);
    });

    return () => timeouts.forEach(clearTimeout);
  }, []);

  return (
    <section className="py-20 bg-zinc-50 dark:bg-[#050505] border-y border-zinc-200 dark:border-zinc-800">
      <div className="container mx-auto px-4">
        <div className="flex flex-col lg:flex-row items-center gap-16">
          <div className="flex-1 space-y-8">
            <h2 className="text-3xl md:text-4xl font-bold text-zinc-900 dark:text-white">
              Feel the power of <br/>
              <span className="text-blue-600 dark:text-blue-500">Systematic Execution</span>
            </h2>
            <p className="text-lg text-zinc-600 dark:text-zinc-400 leading-relaxed">
              Tesser isn't just a library; it's a runtime environment. Watch your strategies react to market data in microseconds, with full observability into every decision, order, and fill.
            </p>
            <div className="grid grid-cols-2 gap-6">
              <div>
                <h4 className="text-2xl font-bold text-zinc-900 dark:text-white mb-1">~15<span className="text-sm text-zinc-500 ml-1">μs</span></h4>
                <p className="text-sm text-zinc-500">Internal Latency</p>
              </div>
              <div>
                <h4 className="text-2xl font-bold text-zinc-900 dark:text-white mb-1">100<span className="text-sm text-zinc-500 ml-1">%</span></h4>
                <p className="text-sm text-zinc-500">Rust Safety</p>
              </div>
            </div>
          </div>

          <div className="flex-1 w-full">
            <div className="relative rounded-xl overflow-hidden bg-[#0d1117] border border-zinc-800 shadow-2xl">
              <div className="flex items-center gap-2 px-4 py-3 bg-white/5 border-b border-white/5">
                <div className="flex gap-1.5">
                  <div className="w-3 h-3 rounded-full bg-red-500/20 border border-red-500/50" />
                  <div className="w-3 h-3 rounded-full bg-yellow-500/20 border border-yellow-500/50" />
                  <div className="w-3 h-3 rounded-full bg-green-500/20 border border-green-500/50" />
                </div>
                <div className="ml-4 text-xs font-mono text-zinc-500 flex items-center gap-2">
                  <Terminal className="size-3" /> tesser-cli live run --exec live
                </div>
              </div>
              <div ref={containerRef} className="p-6 font-mono text-sm h-[300px] overflow-hidden flex flex-col justify-end">
                {lines.map((line, i) => (
                  <motion.div 
                    key={i}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    className={cn(
                      "mb-2",
                      line.includes("WARN") ? "text-yellow-400" :
                      line.includes("EXEC") ? "text-blue-400" :
                      line.includes("✓") ? "text-green-400" :
                      "text-zinc-400"
                    )}
                  >
                    <span className="text-zinc-600 mr-3">{new Date().toLocaleTimeString()}</span>
                    {line}
                  </motion.div>
                ))}
                <div className="w-2 h-4 bg-blue-500 animate-pulse mt-1" />
              </div>
              {/* Glass overlay effect */}
              <div className="absolute inset-0 bg-gradient-to-t from-blue-500/5 to-transparent pointer-events-none" />
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function BentoGrid() {
  return (
    <section className="py-24 bg-white dark:bg-black">
      <div className="container mx-auto px-4">
        <div className="text-center max-w-3xl mx-auto mb-16">
          <h2 className="text-3xl md:text-4xl font-bold text-zinc-900 dark:text-white mb-4">
            Architecture that scales with your AUM
          </h2>
          <p className="text-zinc-600 dark:text-zinc-400">
            Move from backtesting on your laptop to high-frequency trading on the cloud without changing your strategy code.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 h-auto md:h-[600px]">
          {/* Large item */}
          <BentoItem 
            className="md:col-span-2 md:row-span-1" 
            title="Unified Event Loop"
            description="A single, high-performance event loop handles market data ingestion, signal processing, and order execution. This ensures deterministic behavior between backtesting and live trading."
            icon={<Activity className="size-8 text-blue-500" />}
            bgImage="/grid.svg" // Placeholder
          >
          </BentoItem>

          <BentoItem 
            title="Risk Engine" 
            description="Pre-trade checks and portfolio-level drawdown protection prevent catastrophic losses."
            icon={<ShieldCheck className="size-6 text-green-500" />}
          />
          <BentoItem 
            title="Extensible Connectors" 
            description="Write once, trade anywhere. Implement simple traits to support new exchanges."
            icon={<Globe className="size-6 text-purple-500" />}
          />
          <BentoItem 
            title="State Persistence" 
            description="SQLite-backed state management ensures you can recover from process restarts."
            icon={<Layers className="size-6 text-amber-500" />}
          />
        </div>
      </div>
    </section>
  );
}

function BentoItem({ 
  className, title, description, icon, children, bgImage 
}: { 
  className?: string, title: string, description: string, icon: React.ReactNode, children?: React.ReactNode, bgImage?: string 
}) {
  return (
    <motion.div 
      whileHover={{ y: -5 }}
      className={cn(
        "group relative overflow-hidden rounded-3xl border border-zinc-200 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-900/50 p-8 flex flex-col justify-between hover:border-zinc-300 dark:hover:border-zinc-700 transition-all",
        className
      )}
    >
      {bgImage && <div className="absolute inset-0 opacity-5 dark:opacity-10 bg-[url('/grid.svg')] bg-center pointer-events-none" />}
      
      <div>
        <div className="mb-4 inline-flex p-3 rounded-xl bg-white dark:bg-zinc-800 shadow-sm border border-zinc-100 dark:border-zinc-700">
          {icon}
        </div>
        <h3 className="text-xl font-bold text-zinc-900 dark:text-white mb-2">{title}</h3>
        <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">{description}</p>
      </div>
      {children}
    </motion.div>
  );
}
function RoadmapSection() {
  const items = [
    { status: "done", title: "Core Engine", desc: "Event loop, Backtester, Paper Trading" },
    { status: "done", title: "Bybit Connector", desc: "Linear Perpetuals, WebSocket Streams" },
    { status: "current", title: "Live Trading CLI", desc: "TUI, Prometheus Metrics, Webhooks" },
    { status: "future", title: "WASM Plugins", desc: "Hot-reload strategies, Multi-language support" },
    { status: "future", title: "Cloud Orchestrator", desc: "Kubernetes operator for massive scale" },
  ];

  return (
    <section className="py-24 border-t border-zinc-200 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-950">
      <div className="container mx-auto px-4">
        <div className="mb-16 text-center">
          <h2 className="text-3xl font-bold text-zinc-900 dark:text-white">The Roadmap</h2>
          <p className="text-zinc-500 mt-2">We are building the future of open-source quant infrastructure.</p>
        </div>
        
        <div className="relative max-w-4xl mx-auto">
          {/* Vertical Line */}
          <div className="absolute left-8 top-0 bottom-0 w-px bg-zinc-200 dark:bg-zinc-800 md:left-1/2" />
          
          <div className="space-y-12">
            {items.map((item, i) => (
              <motion.div 
                key={i}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: i * 0.1 }}
                className={cn(
                  "relative flex items-center md:justify-between",
                  i % 2 === 0 ? "md:flex-row-reverse" : ""
                )}
              >
                <div className="absolute left-8 md:left-1/2 -translate-x-1/2 w-4 h-4 rounded-full border-4 border-zinc-50 dark:border-black z-10 bg-white dark:bg-zinc-800">
                   <div className={cn(
                     "w-full h-full rounded-full",
                     item.status === "done" ? "bg-green-500" :
                     item.status === "current" ? "bg-blue-500 animate-pulse" : "bg-zinc-300 dark:bg-zinc-700"
                   )} />
                </div>
                
                <div className="pl-20 md:pl-0 md:w-[45%]">
                  <div className="p-6 rounded-2xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 shadow-sm hover:shadow-md transition-shadow">
                    <div className="flex items-center gap-2 mb-2">
                      {item.status === 'current' && (
                        <span className="px-2 py-0.5 rounded-full bg-blue-500/10 text-blue-600 dark:text-blue-400 text-[10px] font-bold uppercase tracking-wider border border-blue-500/20">
                          Current Focus
                        </span>
                      )}
                    </div>
                    <h3 className="text-lg font-bold text-zinc-900 dark:text-white">{item.title}</h3>
                    <p className="text-zinc-600 dark:text-zinc-400 text-sm mt-1">{item.desc}</p>
                  </div>
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

function CTASection() {
  return (
    <section className="py-32 relative overflow-hidden">
      <div className="absolute inset-0 bg-blue-600 dark:bg-blue-950">
         <div className="absolute inset-0 bg-[url('/grid.svg')] opacity-10" />
         <div className="absolute -top-24 -right-24 w-96 h-96 bg-blue-400/30 blur-3xl rounded-full" />
         <div className="absolute -bottom-24 -left-24 w-96 h-96 bg-purple-400/30 blur-3xl rounded-full" />
      </div>
      
      <div className="container mx-auto px-4 relative z-10 text-center">
        <h2 className="text-4xl md:text-5xl font-bold text-white mb-6">
          Ready to professionalize your trading?
        </h2>
        <p className="text-blue-100 text-lg max-w-2xl mx-auto mb-10">
          Join the community of quants building the next generation of trading infrastructure with Tesser.
        </p>
        <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
          <Link 
            href="/docs"
            className="px-8 py-4 rounded-full bg-white text-blue-900 font-bold text-lg hover:bg-blue-50 transition-colors shadow-lg"
          >
            Get Started Now
          </Link>
          <Link 
            href="/docs/getting-started"
            className="px-8 py-4 rounded-full border border-white/30 text-white font-semibold text-lg hover:bg-white/10 transition-colors"
          >
            Read the Docs
          </Link>
        </div>
      </div>
    </section>
  );
}

export default function HomePage() {
  return (
    <main className="flex flex-col min-h-screen bg-white dark:bg-black transition-colors duration-300 font-sans selection:bg-blue-500/30">
      <HeroSection />
      <TerminalSimulation />
      <BentoGrid />
      <RoadmapSection />
      <CTASection />
    </main>
  );
}
