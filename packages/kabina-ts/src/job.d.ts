import { MapDependenciesToArguments } from "./deps.d.ts";
import { InvocationConfig } from "./index.d.ts";

export interface JobConfig<D, O> {
  name: string,
  run: JobRuner<D, O>
  deps?: D
}

export interface Job<O> {
  __job: 'Job'
}

export function job<O, D = []>(req: JobConfig<D, O>): Job<O>


export interface JobFunctionRunner<D, O> {
  func: (input: MapDependenciesToArguments<D>) => O;
}

export interface JobBinaryRunner<D, O> {
  binary: (dependencies: MapDependenciesToArguments<D>) => InvocationConfig<O>
}

export type JobRuner<D, O> = JobFunctionRunner<D, O> | JobBinaryRunner<D, O>;
