import Image from 'next/image';
import Link from 'next/link';
import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';
import { SITE_NAME } from './metadata';

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: <NavLogo />,
      url: '/',
      children: (
        <Link
          href="/docs"
          className="hidden text-sm font-medium text-fd-muted-foreground ring-offset-background transition-colors hover:text-fd-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-fd-primary focus-visible:ring-offset-2 md:inline-flex"
        >
          Docs
        </Link>
      ),
    },
    themeSwitch: {
      enabled: true,
      mode: 'light-dark',
    },
    githubUrl: 'https://github.com/tesserspace/tesser',
  };
}

function NavLogo() {
  return (
    <span className="inline-flex items-center gap-2 font-semibold text-fd-foreground">
      <Image
        src="/tesser-logo.png"
        alt="Tesser"
        width={28}
        height={28}
        className="rounded"
        priority
      />
      <span>{SITE_NAME}</span>
    </span>
  );
}
