import * as env from 'universal-env';

if (env.isWeb) {
  console.log('This is web');
} else if (env.isWeex) {
  console.log('This is weex');
} else {
  console.log('others');
}

