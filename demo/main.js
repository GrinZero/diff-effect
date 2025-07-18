import { analyzeDiffWithCode } from "../src/index.ts";
import fs from "fs";

const newCode = fs.readFileSync("./demo/new.txt", "utf-8");
const oldCode = fs.readFileSync("./demo/old.txt", "utf-8");

const diff = analyzeDiffWithCode(oldCode, newCode);
console.log("diff", diff);
