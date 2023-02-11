import { fileGroup } from 'kabina'

fileGroup({
  name: "Kabina::Files",
  root: "packages",
  items: [
    { pattern: '**/*.rs' },
  ]
})
