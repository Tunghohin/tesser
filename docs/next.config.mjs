import fs from 'node:fs';
import path from 'node:path';
import { createMDX } from 'fumadocs-mdx/next';

const withMDX = createMDX();

function readWorkspaceVersion() {
  try {
    const cargoPath = path.resolve(process.cwd(), '../Cargo.toml');
    const contents = fs.readFileSync(cargoPath, 'utf8');
    const match = contents.match(/^\s*version\s*=\s*"([^"]+)"/m);
    return match?.[1] ?? '0.0.0';
  } catch {
    return '0.0.0';
  }
}

const workspaceVersion = readWorkspaceVersion();

/** @type {import('next').NextConfig} */
const config = {
  reactStrictMode: true,
  env: {
    NEXT_PUBLIC_TESSER_VERSION:
      process.env.NEXT_PUBLIC_TESSER_VERSION ?? workspaceVersion,
  },
};

export default withMDX(config);
