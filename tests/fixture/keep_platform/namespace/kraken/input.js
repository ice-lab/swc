import * as env from 'universal-env';

if (env.isKraken) {
  console.log('This is kraken');
} else if (env.isWeex) {
  console.log('This is weex');
} else {
  console.log('others');
}

