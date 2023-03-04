import type {
  fileGroup as FileGroupFunc,
  transform as TransformFunc,
  pack as PackageFunc,
  server as ServerFunc,
  toolchain as ToolchainFunc,
  FileGroupConfig,
  TransformConfig,
  FileGroup,
  Transform,
  PackageConfig,
  ServerConfig,
  ToolchainConfig,
} from "kabina";

declare interface Deno {
  core: {
    ops: {
      file_group: (cfg: FileGroupConfig) => number;
      transform: (cfg: TransformConfigRuntime) => number;
      package: (cfg: PackageConfig) => number;
      server: (cfg: ServerConfig) => number;
      toolchain: (cfg: ToolchainConfig) => number;
    };
  };
}

declare const Deno: Deno;

export interface Bind {
  cb?: (file: string) => string;
}

export const up = 3;

export default function caller(this: Bind | any, levelUp = up) {
  const err = new Error();
  const stack = err.stack?.split("\n")[levelUp];
  if (stack) {
    return getFile.bind(this)(stack);
  }
}

export function getFile(this: Bind | any, stack: string) {
  stack = stack.substr(stack.indexOf("at ") + 3);
  if (!stack.startsWith("file://")) {
    stack = stack.substr(stack.lastIndexOf("(") + 1);
  }
  const path = stack.split(":");
  let file = `${path[0]}:${path[1]}`;

  if ((this as Bind)?.cb) {
    const cb = (this as Bind).cb as any;
    file = cb(file);
  }
  return file;
}

export const fileGroup: typeof FileGroupFunc = (
  fileGroupConfig: FileGroupConfig & { module?: string }
) => {
  fileGroupConfig.module = caller();
  const id: number = Deno.core.ops.file_group(fileGroupConfig);
  return {
    kind: "FileGroup",
    id,
  };
};

// deno-lint-ignore no-explicit-any
type Dependency = FileGroup | Transform<any>;

interface TransformConfigRuntime {
  name: string;
  module: string | undefined;
  runner: number;
  // deno-lint-ignore no-explicit-any
  input: any;
  // deno-lint-ignore no-explicit-any
  dependencies: any;
}

let runtimeFunctionsSeq = 0;

// deno-lint-ignore ban-types
const transforms = new Map<number, Function>();

(globalThis as any).__transforms = transforms;

export const transform: typeof TransformFunc = <I, D, O>(
  transformConfig: TransformConfig<I, D, O>
) => {
  // const runnerId = runtimeFunctionsSeq++;

  const config: TransformConfigRuntime = {
    name: transformConfig.name,
    module: caller(),
    input: transformConfig.input,
    dependencies: transformConfig.dependencies || null,
    runner: 0,
  };

  const id: number = Deno.core.ops.transform(config);

  transforms.set(id, transformConfig.run);

  return {
    kind: "Transform",
    id,
  };
};

export const pack: typeof PackageFunc = (config: PackageConfig) => {
  const id: number = Deno.core.ops.package(config);

  return {
    kind: "Package",
    id
  }
}

export const server: typeof ServerFunc = (config: ServerConfig) => {
  const id: number = Deno.core.ops.server(config);

  return {
    kind: "Server",
    id
  }
}

export const toolchain: typeof ToolchainFunc = (config: ToolchainConfig) => {
  const id: number = Deno.core.ops.toolchain(config);

  return {
    kind: "Toolchain",
    id
  }
}
