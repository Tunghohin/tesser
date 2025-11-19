import fs from 'node:fs';
import path from 'node:path';

let cachedVersion: string | null = null;

export function getWorkspaceVersion(): string {
  if (cachedVersion) return cachedVersion;

  const cargoPath = path.resolve(process.cwd(), '../Cargo.toml');

  try {
    const contents = fs.readFileSync(cargoPath, 'utf8');
    const match = contents.match(/^\s*version\s*=\s*"([^"]+)"/m);
    cachedVersion = match?.[1] ?? '0.0.0';
  } catch {
    cachedVersion = '0.0.0';
  }

  return cachedVersion;
}
