
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
  kind: "FileGroup",
  id: number
}

export function fileGroup(req: FileGroupConfig): FileGroup

export type Dependency = FileGroup | Transform<any>;

export interface Input {
  __input?: "Input"
}

export interface JobConfig<D, O> {
  name: string,
  run: JobRuner<D, O>
  deps?: D
}

export interface Job<O> {
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

export interface TransformConfig<I, D, O> {
  name: string,
  input: I,
  run: TransformRuner<I, D, O>
  dependencies?: D
}

export interface Transform<O> {
  kind: 'Transform',
  id: number
}

export type ArrayLike<T> = T | T[]
export type MapLike<T> = T | T[] | { [key: string]: T }

export function transform<I extends ArrayLike<Dependency>, D extends MapLike<Dependency>, O>(transform: TransformConfig<I, D, O>): Transform<O>;

export interface TransformBinaryRunner<I, D, O> {
  binary: (input: MapDependenciesToArguments<I>, dependencies: MapDependenciesToArguments<D>) => InvocationConfig<O>
}

export type TransformRuner<I, D, O> = (input: MapDependenciesToArguments<I>, dependencies: MapDependenciesToArguments<D>) => O;

export type MapDependenciesToArguments<D> =
  D extends [...infer T] ? { [P in keyof T]: MapDependencyToArgument<T[P]> } :
  D extends FileGroup ? MapDependencyToArgument<D> :
  never;

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

export interface JobFunctionRunner<D, O> {
  func: (input: MapDependenciesToArguments<D>) => O;
}

export interface JobBinaryRunner<D, O> {
  binary: (dependencies: MapDependenciesToArguments<D>) => InvocationConfig<O>
}

export interface InvocationConfig<O> extends ExternalProcessConfig {

}

export type JobRuner<D, O> = JobFunctionRunner<D, O> | JobBinaryRunner<D, O>;

export interface File {
  __file: "File"
}

export function write(path: string, content: string): File

export function reportStatus(status: 'ready' | 'building' | 'failed'): void;

export interface ExternalProcessConfig {
  command: string,
  arguments?: string[],
  env?: { [key: string]: string | undefined },
  processLogs?: {
    stdout?: (line: any) => void;
  }
}

export interface RouteConfig {

}

export interface BundleConfig {
  name: "Application"
  items: [{ prefix: string, content: Dependency }]
}

export interface Bundle {
  kind: 'Bundle'
}

export function bundle(config: BundleConfig): Bundle;

export interface ServerConfig {
  name: string
  routes: { [key: string]: RouteConfig }
}

export interface Server {
  kind: 'Server'
}

export function server(config: ServerConfig): Server;