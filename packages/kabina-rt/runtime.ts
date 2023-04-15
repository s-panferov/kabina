import type {
  binary as BinaryFunc,
  BinaryConfig,
  collection as CollectionFunc,
  CollectionConfig,
  FileGroup,
  fileGroup as FileGroupFunc,
  FileGroupConfig,
  server as ServerFunc,
  ServerConfig,
  service as ServiceFunc,
  ServiceConfig,
  Transform,
  transform as TransformFunc,
  TransformConfig,
} from "kabina";

declare interface Deno {
  core: {
    ops: {
      file_group: (cfg: FileGroupConfig) => number;
      transform: (cfg: TransformConfigRuntime) => number;
      collection: (cfg: CollectionConfig) => number;
      server: (cfg: ServerConfig) => number;
      service: (cfg: ServiceConfig) => number;
      binary: (cfg: BinaryConfig) => number;
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
  fileGroupConfig: FileGroupConfig & { module?: string },
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

// deno-lint-ignore ban-types
const transforms: { [key: number]: Function } = {};

export const __transforms = transforms;

export const transform: typeof TransformFunc = <I, D, O>(
  transformConfig: TransformConfig<I, D, O>,
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

  transforms[id] = transformConfig.run;

  return {
    kind: "Transform",
    id,
  };
};

export const collection: typeof CollectionFunc = (config: CollectionConfig) => {
  const id: number = Deno.core.ops.collection(config);

  return {
    kind: "Collection",
    id,
  };
};

export const server: typeof ServerFunc = (config: ServerConfig) => {
  const id: number = Deno.core.ops.server(config);

  return {
    kind: "Server",
    id,
  };
};

export const binary: typeof BinaryFunc = (config: BinaryConfig) => {
  const id: number = Deno.core.ops.binary(config);

  return {
    kind: "Binary",
    id,
  };
};

export const service: typeof ServiceFunc = (config: ServiceConfig) => {
  const id: number = Deno.core.ops.service(config);

  return {
    kind: "Service",
    id,
  };
};
