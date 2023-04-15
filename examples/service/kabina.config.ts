import { binary, service } from "kabina";

service({
  name: "PhotoFrame",
  binary: binary({
    name: "PhotoFrame",
    runtime: {
      kind: "native",
      executable: "true",
    },
  }),
});
