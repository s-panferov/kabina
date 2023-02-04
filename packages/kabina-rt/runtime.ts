import type { FileGroup, fileGroup as FileGroupFunc, FileGroupConfig } from 'kabina'

export const fileGroup: typeof FileGroupFunc = (fileGroupConfig) => {
  const id: number = Deno.core.ops.file_group(fileGroupConfig);
  return {
    __fileGroup: "FileGroup",
    id
  }
}