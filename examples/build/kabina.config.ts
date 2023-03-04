import { fileGroup, transform, server, bundle, toolchain } from 'kabina'

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

const postcss = toolchain({
  binary: "postcss",
  runner: "node"
})

const postCSS = transform({
  name: "PostCSS",
  input: cssFiles,
  dependencies: { postcss },
  run: (ctx, { postcss }) => {
    postcss.invoke([ctx.fileName])
  }
})

const appBundle = bundle({
  name: "Application",
  items: [
    // { prefix: "", content: postCSS },
    { prefix: "", content: postCSS }
  ]
})

const appServer = server({
  name: "Kabina::Server",
  routes: {
    '*': appBundle
  }
})
