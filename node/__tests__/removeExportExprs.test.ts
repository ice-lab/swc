import { transformSync } from '../../node';

describe('swc remove export expressions', () => {
  it('should preserve getConfig function', () => {
    const originalCode = `import fs from 'fs'
    import other from 'other'

    const [a, b, ...rest] = fs.promises
    const [foo, bar] = other

    export async function getData() {
      a
      b
      rest
      bar
    }

    export function getConfig() {
    }

    export default function Home() {
      return <div />
    }`;

    const { code } = transformSync(originalCode, {
      sourceMaps: false,
      jsc: {
        parser: {
          syntax: 'ecmascript',
          jsx: true,
        },
        target: 'es2016',
      },
      removeExportExprs: ['getData', 'default'],
    });

    expect(code).toEqual(`import other from 'other';
const [foo] = other;
export function getConfig() {}
export default function() {};
`);
  });
});
