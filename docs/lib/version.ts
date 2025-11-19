let cachedVersion: string | null = null;

export function getWorkspaceVersion(): string {
  if (cachedVersion) return cachedVersion;

  cachedVersion = process.env.NEXT_PUBLIC_TESSER_VERSION ?? '0.0.0';
  return cachedVersion;
}
