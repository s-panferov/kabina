
export interface FilePattern {
  pattern: string,
  version?: "time" | "hash"
}

export interface FileGroupConfig {
  name: string,
  root?: string,
  items: (string | FilePattern)[]
}

export interface FileGroup extends Input {
  __fileGroup: 'FileGroup'
}

export function fileGroup(req: FileGroupConfig): FileGroup

export interface Dependency {
  __dependency?: "Dependency"
}

export interface Input {
  __input?: "Input"
}

export interface JobConfig<D, O> {
  name: string,
  run: JobRuner<D, O>
  deps?: D
}

export interface Job<O> extends Dependency {
  __job: 'Job'
}

// declare const InvalidTypeSymbol = Symbol(`Invalid type`);
// // eslint-disable-next-line @typescript-eslint/no-unused-vars
// export type invalid<ErrorMessage> =
//   | ((
//     invalidType: typeof InvalidTypeSymbol,
//     ..._: typeof InvalidTypeSymbol[]
//   ) => typeof InvalidTypeSymbol)
//   | null
//   | undefined;

export function job<O, D = []>(req: JobConfig<D, O>): Job<O>

export type MapDependenciesToArguments<D> = D extends [...infer T] ?
  { [P in keyof T]: MapDependencyToArgument<T[P]> } : never;

export interface FileMetadata {
  fileName: string
}

export interface FileContent extends FileMetadata {
  buffer: ArrayBuffer
}

export type MapDependencyToArgument<D> =
  D extends FileGroup ? FileMetadata :
  D extends Job<infer O> ? MapDependencyToArgument<O> :
  never;

export interface FunctionRunner<D, O> {
  func: (input: MapDependenciesToArguments<D>) => O;
}

export type FunctionOutput = string | number | File

export interface BinaryRunner<D, O> {
  binary: (dependencies: MapDependenciesToArguments<D>) => InvocationConfig<O>
}

export interface InvocationConfig<O> extends ExternalProcessConfig {

}

export type JobRuner<D, O> = FunctionRunner<D, O> | BinaryRunner<D, O>;

export interface File {
  __file: "File"
}

export function write(path: string, content: string): File

export interface LogLine {

}

export function reportStatus(status: 'ready' | 'building' | 'failed'): void;

export interface ExternalProcessConfig {
  command: string,
  arguments?: string[],
  env?: { [key: string]: string | undefined },
  processLogs?: {
    stdout?: (line: LogLine) => void;
  }
}

