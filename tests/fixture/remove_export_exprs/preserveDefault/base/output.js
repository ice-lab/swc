import fs from 'fs';
const [a] = fs.promises;
export default class Home {
    constructor(){
        console.log(a);
    }
};
