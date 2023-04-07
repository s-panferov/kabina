
export interface ServiceConfig {
  name: string
  runtime: ServiceRuntime
}

export interface Service {
  kind: 'Service'
}

export function service(config: ServiceConfig): Service;

export type ServiceRuntime = BinaryRuntime 

export interface BinaryRuntime {
  kind: 'binary',
  executable: string
}