import { Binary } from "./binary.d.ts"

export interface ServiceConfig {
  name: string
  binary: Binary
}

export interface Service {
  kind: 'Service'
}

export function service(config: ServiceConfig): Service;
