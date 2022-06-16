import fs from 'fs'
import other from 'other'

const [a, b, ...rest] = fs.promises
const [bar] = other

export async function getData() {
  a
  b
  rest
}

export function getConfig() {
}

export default () => {
  return bar;
}
