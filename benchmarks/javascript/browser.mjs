import { affine } from "/.zop/benchmarks/javascript/affine.mjs";
import { F64 } from "/.zop/benchmarks/javascript/topcoat.mjs";
import { benchmark, report } from "./workload.mjs";

const output = report(benchmark(affine, F64));
document.querySelector("pre").textContent = output;
document.documentElement.dataset.benchmark = "complete";
