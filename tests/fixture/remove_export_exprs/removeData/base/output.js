import fs from 'fs';
const [a] = fs.promises;
export function getConfig() {
    console.log(1);
}
export default class Home {
    constructor(){
        console.log(a);
    }
};
