const values = Float64Array.from({ length: 1024 }, (_, index) => index / 7 + 0.25);
const iterations = 20_000_000;
const samples = 9;

export function benchmark(affine, F64) {
    function direct(value, scale, bias) {
        return value * scale + bias;
    }

    function topcoat(value, scale, bias) {
        return new F64(value).mul(new F64(scale)).add(new F64(bias)).dehydrate();
    }

    function executeZop(count) {
        let checksum = 0;
        for (let index = 0; index < count; index += 1) {
            checksum += affine(values[index & 1023], 1.25, 0.5);
        }
        return checksum;
    }

    function executeDirect(count) {
        let checksum = 0;
        for (let index = 0; index < count; index += 1) {
            checksum += direct(values[index & 1023], 1.25, 0.5);
        }
        return checksum;
    }

    function executeTopcoat(count) {
        let checksum = 0;
        for (let index = 0; index < count; index += 1) {
            checksum += topcoat(values[index & 1023], 1.25, 0.5);
        }
        return checksum;
    }

    const cases = [
        ["zop", executeZop],
        ["direct-js", executeDirect],
        ["topcoat", executeTopcoat],
    ];
    const expected = executeDirect(values.length);
    for (const [name, execute] of cases) {
        const actual = execute(values.length);
        if (actual !== expected) throw new Error(`${name} checksum: ${actual}`);
        execute(1_000_000);
    }

    const elapsed = new Map(cases.map(([name]) => [name, []]));
    for (let sample = 0; sample < samples; sample += 1) {
        for (let offset = 0; offset < cases.length; offset += 1) {
            const [name, execute] = cases[(sample + offset) % cases.length];
            const started = performance.now();
            globalThis.__zopBenchmarkChecksum = execute(iterations);
            elapsed.get(name).push(performance.now() - started);
        }
    }

    return Object.fromEntries(cases.map(([name]) => [name, median(elapsed.get(name))]));
}

export function report(results) {
    return [
        `zop        ${results.zop.toFixed(3)} ms`,
        `direct-js  ${results["direct-js"].toFixed(3)} ms`,
        `topcoat    ${results.topcoat.toFixed(3)} ms`,
        `zop/direct  ${(results.zop / results["direct-js"]).toFixed(3)}x`,
        `zop/topcoat ${(results.zop / results.topcoat).toFixed(3)}x`,
    ].join("\n");
}

function median(values) {
    const ordered = values.toSorted((left, right) => left - right);
    return ordered[Math.floor(ordered.length / 2)];
}
