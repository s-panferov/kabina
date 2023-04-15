
export interface Binary {
  kind: 'Binary'
}

export interface BinaryConfig {
  name: string,
  runtime: BinaryRuntime
}

export type BinaryRuntime = BinaryNative 

export interface BinaryNative {
  kind: 'native',
  executable: string
}

export interface BinaryRunner {
  invoke(arguments: string[]): void
}

export function binary(config: BinaryConfig): Binary;