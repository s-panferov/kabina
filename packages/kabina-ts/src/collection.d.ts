import { Dependency } from "./deps.d.ts";

export interface CollectionConfig {
  name: string
  items: { prefix: string, content: Dependency }[]
}

export interface Collection {
  kind: 'Collection'
}

export function collection(config: CollectionConfig): Collection;
