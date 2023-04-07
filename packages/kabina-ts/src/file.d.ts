
export interface FilePattern {
  pattern: string,
  version?: "time" | "hash"
}

export interface FileGroupConfig {
  name: string,
  root?: string,
  items: (string | FilePattern)[]
}

export interface FileGroup {
  kind: "FileGroup",
  id: number
}

export function fileGroup(req: FileGroupConfig): FileGroup

export interface FileMetadata {
  fileName: string
}

export interface FileContent extends FileMetadata {
  buffer: ArrayBuffer
}

export interface File {
  __file: "File"
}
