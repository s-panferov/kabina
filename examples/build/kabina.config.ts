import { fileGroup, transform, server, bundle } from 'kabina'

const cssFiles = fileGroup({
  name: "css",
  items: [
    { pattern: 'src/css/**/*.css' }
  ]
})

const jsFiles = fileGroup({
  name: "js",
  root: "src",
  items: [
    { pattern: '**/*.js' }
  ]
})

const PostCSS = transform({
  name: "PostCSS",
  input: cssFiles,
  run: async (ctx) => {
  }
})

const appBundle = bundle({
  name: "Application",
  items: [
    { prefix: "", content: PostCSS }
  ]
})

const appServer = server({
  name: "Kabina::Server",
  routes: {
    '*': appBundle
  }
})
