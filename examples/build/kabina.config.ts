import { fileGroup, transform, server, bundle } from 'kabina'

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

const postCSS = transform({
  name: "PostCSS",
  input: cssFiles,
  run: (ctx) => {
    return true
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
