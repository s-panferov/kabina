
export interface ServerConfig {
  name: string
  routes: { [key: string]: RouteConfig }
}

export interface Server {
  kind: 'Server'
}

export function server(config: ServerConfig): Server;


export interface RouteConfig {

}
