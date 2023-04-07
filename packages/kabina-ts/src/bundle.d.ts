import { MapLike } from "./deps.d.ts";
import { MapDependenciesToArguments } from "./deps.d.ts";
import { Dependency } from "./deps.d.ts";
import { Transform } from "./transform.d.ts";

export interface BundleConfig<I, D, O> {
  name: string,
  input: I,
  run: BundleRuner<I, D, O>
  dependencies?: D
}

export function bundle<I extends ArrayLike<Dependency>, D extends MapLike<Dependency>, O>(transform: BundleConfig<I, D, O>): Transform<O>;

export type BundleRuner<I, D, O> = (input: MapDependenciesToArguments<I>, dependencies: MapDependenciesToArguments<D>) => O;