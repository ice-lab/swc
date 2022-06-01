import React from 'react';
import fs from 'fs';
import other from 'other';

const [a, b, ...rest] = fs.promises;
const [bar] = other;

export async function getData() {
  a;
  b;
  rest;
  bar;
}

class Foo {}


export default function Home() {
  return <div />;
}
