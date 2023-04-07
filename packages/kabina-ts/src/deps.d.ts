import { FileGroup, FileMetadata } from "./file.d.ts";
import { Job } from "./job.d.ts";
import { ToolchainRunner } from "./toolchain.d.ts";
import { Toolchain } from "./toolchain.d.ts";

export type Dependency = FileGroup | Transformer<any> | Toolchain;

export type ArrayLike<T> = T | T[]
export type MapLike<T> = T | T[] | { [key: string]: T }

export type MapDependenciesToArguments<D> =
  D extends [...infer T] ? { [P in keyof T]: MapDependencyToArgument<T[P]> } :
  D extends { [K in keyof D]: Dependency } ? { [P in keyof D]: MapDependencyToArgument<D[P]> } :
  MapDependencyToArgument<D>;


export type MapDependencyToArgument<D> =
  D extends FileGroup ? FileMetadata :
  D extends Job<infer O> ? MapDependencyToArgument<O> :
  D extends Toolchain ? ToolchainRunner :
  never;
