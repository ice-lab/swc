function e(c,a){(null==a||a>c.length)&&(a=c.length);for(var b=0,d=new Array(a);b<a;b++)d[b]=c[b];return d}function f(a){if(Array.isArray(a))return a}function g(c,d,e,f,g,h,i){try{var a=c[h](i),b=a.value}catch(j){e(j);return}a.done?d(b):Promise.resolve(b).then(f,g)}function h(){throw new TypeError("Invalid attempt to destructure non-iterable instance.\\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method.")}function i(a,c){if(a){if("string"==typeof a)return e(a,c);var b=Object.prototype.toString.call(a).slice(8,-1);if("Object"===b&&a.constructor&&(b=a.constructor.name),"Map"===b||"Set"===b)return Array.from(b);if("Arguments"===b||/^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(b))return e(a,c)}}import j from"regenerator-runtime";import c from"fs";import d from"other";var a=function(a){return f(a)||function(a){if("undefined"!=typeof Symbol&&null!=a[Symbol.iterator]||null!=a["@@iterator"])return Array.from(a)}(a)||i(a,i)||h()}(c.promises);a[0],a[1],a.slice(2);var b=function(a,b){return f(a)||function(b,e){var f,g,a=null==b?null:"undefined"!=typeof Symbol&&b[Symbol.iterator]||b["@@iterator"];if(null!=a){var c=[],d=!0,h=!1;try{for(a=a.call(b);!(d=(f=a.next()).done)&&(c.push(f.value),!e||c.length!==e);d=!0);}catch(i){h=!0,g=i}finally{try{d||null==a.return||a.return()}finally{if(h)throw g}}return c}}(a,b)||i(a,b)||h()}(d,2);b[0],b[1];export function getStaticProps(){return k.apply(this,arguments)}function k(){var a;return(k=(a=j.mark(function a(){return j.wrap(function(a){for(;;)switch(a.prev=a.next){case 0:case 4:case"end":return a.stop()}},a)}),function(){var b=this,c=arguments;return new Promise(function(e,f){var h=a.apply(b,c);function d(a){g(h,e,f,d,i,"next",a)}function i(a){g(h,e,f,d,i,"throw",a)}d(void 0)})})).apply(this,arguments)}export default function l(){return React.createElement("div",null)}
