import fs from 'fs'

const [a, b, ...rest] = fs.promises

async function t() {
  a
  b
  rest
}

export const getData = t;

export function getConfig() {
  console.log(1)
}