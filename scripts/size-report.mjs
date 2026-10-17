// Prints raw, gzip and Brotli sizes of the shipped JS and WASM files.
// Usage: node scripts/size-report.mjs web/dist
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { brotliCompressSync, constants, gzipSync } from 'node:zlib';

const root = process.argv[2] ?? 'web/dist';
const files = [];
const walk = (dir) => {
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) walk(path);
    else if (/\.(js|wasm)$/.test(name)) files.push(path);
  }
};
walk(root);

const kib = (n) => `${(n / 1024).toFixed(1)} KiB`.padStart(12);
console.log(`${'file'.padEnd(36)}${'raw'.padStart(12)}${'gzip -9'.padStart(12)}${'brotli 11'.padStart(12)}`);
for (const path of files.sort()) {
  const data = readFileSync(path);
  const gz = gzipSync(data, { level: 9 }).length;
  const br = brotliCompressSync(data, {
    params: { [constants.BROTLI_PARAM_QUALITY]: 11 },
  }).length;
  console.log(`${relative(root, path).padEnd(36)}${kib(data.length)}${kib(gz)}${kib(br)}`);
}
