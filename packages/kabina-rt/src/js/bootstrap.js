import * as url from "ext:deno_url/00_url.js";
// import * as webidl from "ext:deno_webidl/00_webidl.js";

Object.defineProperty(globalThis, "URL", {
    value: url.URL,
    writable: true,
    enumerable: false,
    configurable: true,
})
