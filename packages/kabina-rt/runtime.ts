import type { fileGroup as FileGroupFunc, transform as TransformFunc, FileGroupConfig, TransformConfig, FileGroup, Transform } from 'kabina'

declare interface Deno {
  core: {
    ops: {
      file_group: (cfg: FileGroupConfig) => number
      transform: (cfg: TransformConfigRuntime) => number
    }
  }
}

declare const Deno: Deno;

export interface Bind {
  cb?: (file: string) => string;
}

export const up = 3;

export default function caller(this: Bind | any, levelUp = up) {
  const err = new Error();
  const stack = err.stack?.split('\n')[levelUp];
  if (stack) {
    return getFile.bind(this)(stack);
  }
}

export function getFile(this: Bind | any, stack: string) {
  stack = stack.substr(stack.indexOf('at ') + 3);
  if (!stack.startsWith('file://')) {
    stack = stack.substr(stack.lastIndexOf('(') + 1);
  }
  const path = stack.split(':');
  let file = `${path[0]}:${path[1]}`;

  if ((this as Bind)?.cb) {
    const cb = (this as Bind).cb as any;
    file = cb(file);
  }
  return file;
}

export const fileGroup: typeof FileGroupFunc = (fileGroupConfig: FileGroupConfig & { module?: string }) => {
  fileGroupConfig.module = caller()
  const id: number = Deno.core.ops.file_group(fileGroupConfig);
  return {
    kind: "FileGroup",
    id
  }
}

type Dependency = FileGroup | Transform<any>;

interface TransformConfigRuntime {
  name: string,
  module: string | undefined,
  runner: number,
  input: Dependency[]
  deps: Dependency[]
}

let runtimeFunctionsSeq = 0
const runtimeFunctions = new Map<number, WeakRef<Function>>;

function toArray<I>(array: I) {
  if Array.isArray(array) {

  }
}

export const transform: typeof TransformFunc = <I, D, O>(transformConfig: TransformConfig<I, D, O>) => {
  const runnerId = runtimeFunctionsSeq++;

  runtimeFunctions.set(runnerId, new WeakRef(() => { }));

  const config: TransformConfigRuntime = {
    name: transformConfig.name,
    module: caller(),
    input: transformConfig.input,
    deps: transformConfig.deps,
    runner: runnerId
  }

  const id: number = Deno.core.ops.transform(config);
  return {
    kind: "Transform",
    id
  }
}