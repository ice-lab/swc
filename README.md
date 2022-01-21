## @builder/swc

>swc version is 0.115.0

<img src="https://img.shields.io/npm/v/@builder/swc.svg" alt="npm package" />
<img src="https://img.shields.io/npm/dm/@builder/swc.svg" alt="npm downloads" />

Custom swc for [ice.js](https://github.com/alibaba/ice) and [rax-app](https://github.com/raxjs/rax-app).

<i>Customization is achieved through the pass hook of swc.</i>

### Platform Support

- Windows x64
- macOS x64
- macOS aarch64
- Linux aarch64 gnu
- Linux x64 gnu

### API

#### transformSync(code: string, options)

> Return { code: string, map?: string }

Synchronous transform code. You can pass [@swc/core options](https://swc.rs/docs/configuration/compilation) directly.

```js
import { transformSync } from '@builder/swc';

const { code, map } = transformSync('var a = 1;', {
  jsc: {
    parser: {
      syntax: "ecmascript",
    },
    transform: {},
  },
});
```

**options.keepPlatform**

Transform the variables which exported by `universal-env` to bool. The output code will remove other platform code by compressor.

```js
import { transformSync } from '@builder/swc';

// Case 1: import specifier
// Input
var input = `
import { isWeb, isWeex } from 'universal-env';

if (isWeb) {
  console.log('This is web code');
} else {
  console.log('This is weex code');
}
`;

var { code, map } = transformSync(input, {
  jsc: {
    parser: {
      syntax: "ecmascript",
    },
    transform: {},
  },
  keepPlatform: 'web',
});

console.log(code);
/* The output code is:
var isWeb = true;
var isWeex = false;

if (isWeb) {
  console.log('This is web code');
} else {
  console.log('This is weex code');
}
*/

// Case 2: import namespace specifier
// Input
var input = `
import * as env from 'universal-env';

if (env.isWeb) {
  console.log('This is web code');
} else {
  console.log('This is weex code');
}
`;

var { code, map } = transformSync(input, {
  jsc: {
    parser: {
      syntax: "ecmascript",
    },
    transform: {},
  },
  keepPlatform: 'web',
});

console.log(code);
/* The output code is:
var env = {
  isWeb: true
};

if (env.isWeb) {
  console.log('This is web code');
} else {
  console.log('This is weex code');
}
*/
```

#### transform(code: string, options)

> Return Promise<{ code: string, map?: string }>

Asynchronous transform code. The options is the same as `transformSync`.

```js
import { transformSync } from '@builder/swc';

const { code, map } = await transformSync(input, {
  jsc: {
    parser: {
      syntax: "ecmascript",
    },
    transform: {},
  },
  keepPlatform: 'web',
});
```


#### minify

> Return Promise<{ code: string, map?: string }>

It is the same as [@swc/core](https://swc.rs/docs/configuration/minification).

This API is asynchronous and all of parsing, minification, and code generation will be done in background thread.

```js
import { minify } from "@builder/swc";

const { code, map } = await minify(
  "import foo from '@src/app'; console.log(foo)",
  {
    compress: false,
    mangle: true,
  }
);

expect(code).toMatchInlineSnapshot(`"import a from'@src/app';console.log(a);"`);
```

#### minifySync

> { code: string, map?: string }

It is the same as [@swc/core](https://swc.rs/docs/configuration/minification).

```js
import { minifySync } from "@builder/swc";

const { code, map } = minifySync(
  "import foo from '@src/app'; console.log(foo)",
  {
    compress: false,
    mangle: true,
  }
);

expect(code).toMatchInlineSnapshot(`"import a from'@src/app';console.log(a);"`);
```
