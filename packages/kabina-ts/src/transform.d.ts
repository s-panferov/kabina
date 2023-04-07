import { Dependency } from "./deps.d.ts";
import { MapLike } from "./deps.d.ts";
import { MapDependenciesToArguments } from "./deps.d.ts";
import { InvocationConfig } from "./index.d.ts";

export interface TransformBinaryRunner<I, D, O> {
  binary: (input: MapDependenciesToArguments<I>, dependencies: MapDependenciesToArguments<D>) => InvocationConfig<O>
}

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

export type TransformRuner<I, D, O> = (input: MapDependenciesToArguments<I>, dependencies: MapDependenciesToArguments<D>) => O;


export function transform<I extends ArrayLike<Dependency>, D extends MapLike<Dependency>, O>(transform: TransformConfig<I, D, O>): Transform<O>;