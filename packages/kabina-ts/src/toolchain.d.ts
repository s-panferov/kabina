
export interface Toolchain {
  kind: 'Toolchain'
}

export interface ToolchainConfig {
  name: string,
  binary: string,
  runner: 'native' | 'node',
}

export interface ToolchainRunner {
  invoke(arguments: string[]): void
}

export function toolchain(config: ToolchainConfig): Toolchain;