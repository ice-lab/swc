import fs from 'fs'

const [a, b, ...rest] = fs.promises

export async function getData() {
  a
  b
  rest
}
