import Link from 'next/link';

const ctas = [
  { label: 'Read the docs', href: '/docs' },
  { label: 'GitHub', href: 'https://github.com/tesserspace/tesser' },
];

export default function HomePage() {
  return (
    <section className="flex flex-1 flex-col items-center justify-center gap-8 px-6 text-center sm:px-10">
      <div className="space-y-4 max-w-3xl">
        <p className="text-sm uppercase tracking-[0.3em] text-zinc-400">
          Modular Rust trading framework
        </p>
        <h1 className="text-4xl font-semibold sm:text-5xl">
          Operate bots with confidence
        </h1>
        <p className="text-base text-zinc-300 leading-relaxed">
          Tesser bundles a CLI orchestrator, strategy SDK, portfolio services,
          and exchange connectors. Embed it as a library, or invoke it as a CLI,
          and deploy anywhere.
        </p>
      </div>
      <div className="flex flex-wrap items-center justify-center gap-3">
        {ctas.map((cta) => (
          <Link
            key={cta.href}
            href={cta.href}
            className="rounded-full border border-white/20 px-4 py-2 text-sm font-medium text-white transition hover:border-white hover:bg-white/10"
          >
            {cta.label}
          </Link>
        ))}
      </div>
      <p className="text-sm text-zinc-400">
        Production base URL: <code>tesser.space</code> · Local preview:{' '}
        <code>localhost:3000</code>
      </p>
    </section>
  );
}
