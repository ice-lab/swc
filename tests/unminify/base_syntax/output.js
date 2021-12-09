import regeneratorRuntime from 'regenerator-runtime';
import fs from 'fs';
import other from 'other';

function _arrayWithHoles(arr) {
  if (Array.isArray(arr)) return arr;
}
function asyncGeneratorStep(gen, resolve, reject, _next, _throw, key, arg) {
  try {
    var info = gen[key](arg);
    var { value } = info;
  } catch (error) {
    reject(error);
    return;
  }
  if (info.done) {
    resolve(value);
  } else {
    Promise.resolve(value).then(_next, _throw);
  }
}
function _asyncToGenerator(fn) {
  return function () {
    const self = this; const
      args = arguments;
    return new Promise((resolve, reject) => {
      const gen = fn.apply(self, args);
      function _next(value) {
        asyncGeneratorStep(gen, resolve, reject, _next, _throw, 'next', value);
      }
      function _throw(err) {
        asyncGeneratorStep(gen, resolve, reject, _next, _throw, 'throw', err);
      }
      _next(undefined);
    });
  };
}
function _classCallCheck(instance, Constructor) {
  if (!(instance instanceof Constructor)) {
    throw new TypeError('Cannot call a class as a function');
  }
}
function _iterableToArray(iter) {
  if (Symbol.iterator in Object(iter) || Object.prototype.toString.call(iter) === '[object Arguments]') return Array.from(iter);
}
function _iterableToArrayLimit(arr, i) {
  const _arr = [];
  let _n = true;
  let _d = false;
  let _e;
  try {
    for (var _i = arr[Symbol.iterator](), _s; !(_n = (_s = _i.next()).done); _n = true) {
      _arr.push(_s.value);
      if (i && _arr.length === i) break;
    }
  } catch (err) {
    _d = true;
    _e = err;
  } finally {
    try {
      if (!_n && _i['return'] != null) _i['return']();
    } finally {
      if (_d) throw _e;
    }
  }
  return _arr;
}
function _nonIterableRest() {
  throw new TypeError('Invalid attempt to destructure non-iterable instance');
}
function _slicedToArray(arr, i) {
  return _arrayWithHoles(arr) || _iterableToArrayLimit(arr, i) || _nonIterableRest();
}
function _toArray(arr) {
  return _arrayWithHoles(arr) || _iterableToArray(arr) || _nonIterableRest();
}
const _promises = _toArray(fs.promises); const a = _promises[0]; const b = _promises[1]; const
  rest = _promises.slice(2);
const _other = _slicedToArray(other, 2); const foo = _other[0]; const
  bar = _other[1];
function _getStaticProps() {
  _getStaticProps = _asyncToGenerator(regeneratorRuntime.mark(function _callee() {
    return regeneratorRuntime.wrap((_ctx) => {
      while (1) {
        switch (_ctx.prev = _ctx.next) {
          case 0:
            a;
            b;
            rest;
            bar;
          case 4:
          case 'end':
            return _ctx.stop();
        }
      }
    }, _callee);
  }));
  return _getStaticProps.apply(this, arguments);
}
export function getStaticProps() {
  return _getStaticProps.apply(this, arguments);
}
const Foo = function Foo() {
  'use strict';
  _classCallCheck(this, Foo);
};
export default function Home() {
  return (/* #__PURE__ */ React.createElement('div', null));
}
