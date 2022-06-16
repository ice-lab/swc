import fs from 'fs';
import other from 'other';
const [b] = fs.promises;
const [foo] = other;
export default function Home() {
    console.log(b);
    return __jsx("div", null);
};
