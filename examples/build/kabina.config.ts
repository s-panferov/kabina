import { fileGroup, transform, server, pack, toolchain } from 'kabina'

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
  dependencies: { postcss },
  run: (ctx, { postcss }) => {
    return postcss.invoke([ctx.fileName])
  }
})

const appBundle = pack({
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
