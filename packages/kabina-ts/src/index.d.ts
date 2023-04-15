export * from './bundle.d.ts'
export * from './collection.d.ts'
export * from './deps.d.ts'
export * from './file.d.ts'
export * from './job.d.ts'
export * from './server.d.ts'
export * from './binary.d.ts'
export * from './transform.d.ts'
export * from './service.d.ts'

export function write(path: string, content: string): File
export function reportStatus(status: 'ready' | 'building' | 'failed'): void;

export interface InvocationConfig<O> extends ExternalProcessConfig {

}

export interface ExternalProcessConfig {
  command: string,
  arguments?: string[],
  env?: { [key: string]: string | undefined },
  processLogs?: {
    stdout?: (line: any) => void;
  }
}
