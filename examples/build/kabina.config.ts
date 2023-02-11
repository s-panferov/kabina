import { fileGroup, transform } from 'kabina'

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

const tranform = transform({
  name: "PostCSS",
  input: cssFiles,
  run: async (ctx) => {
  }
})

// export const concat = job({
//   name: "concat",
//   deps: [files],
//   run: {
//     func(input) {
//       input
//       return files;
//     }
//   }
// })

// const webpack = job({
//   name: "Webpack",
//   deps: [concat],
//   run: {
//     binary: () => {
//       return {
//         command: 'webpack',
//         arguments: ["webpack.config.js"],
//         processLogs: {
//           stdout: (line) => {
//             reportStatus("ready")
//           }
//         }
//       }
//     }
//   }
// })
