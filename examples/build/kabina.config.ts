import { fileGroup } from 'kabina'
// import { job, fileGroup, reportStatus } from 'kabina'

export const files = fileGroup({
  name: "sources",
  files: [
    { pattern: '' }
  ]
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
