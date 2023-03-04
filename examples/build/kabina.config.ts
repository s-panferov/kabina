import { fileGroup, transform, server, collection, toolchain } from 'kabina'

const cssFiles = fileGroup({
  name: "css",
  items: [
    { pattern: 'src/css/**/*.css' }
  ]
})

const tsFiles = fileGroup({
  name: "js",
  root: "src",
  items: [
    { pattern: '**/*.ts' }
  ]
})

const esbuild = toolchain({
  name: "esbuild",
  binary: "esbuild",
  runner: "native"
})

const postCSS = transform({
  name: "PostCSS",
  input: cssFiles,
  dependencies: { esbuild },
  run: (ctx, { esbuild }) => {
    return true
  }
})

const appCollection = collection({
  name: "Application",
  items: [
    // { prefix: "", content: postCSS },
    { prefix: "", content: postCSS }
  ]
})

const appServer = server({
  name: "Kabina::Server",
  routes: {
    '*': appCollection
  }
})
