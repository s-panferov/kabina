import { service } from 'kabina'

service({
  name: "PhotoFrame",
  runtime: {
    kind: 'binary',
    executable: "frame"
  }
})
