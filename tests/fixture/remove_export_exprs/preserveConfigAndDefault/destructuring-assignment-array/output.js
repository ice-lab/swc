import fs from 'fs';
import other from 'other';
const [a] = fs.promises;
const [foo] = other;
export function getConfig() {}
export default function Home() {
    console.log(a);
    return __jsx("div", null);
}
