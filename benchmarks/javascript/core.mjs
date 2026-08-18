import { pathToFileURL } from "node:url";
import { benchmark, report } from "./workload.mjs";

const [zopPath, topcoatPath] = process.argv.slice(2);
if (!zopPath || !topcoatPath) {
    throw new Error("usage: bun core.mjs <zop.mjs> <topcoat.mjs>");
}

const zop = await import(pathToFileURL(zopPath).href);
const { F64 } = await import(pathToFileURL(topcoatPath).href);

console.log(report(benchmark(zop.affine, F64)));
