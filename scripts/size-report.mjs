// Prints raw, gzip and Brotli sizes of the shipped JS and WASM files, then
// checks them against the download budgets below and fails if one is over.
// Usage: node scripts/size-report.mjs web/dist
//        CUE_SIZE_BUDGET=off node scripts/size-report.mjs web/dist  (report only)
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { brotliCompressSync, constants, gzipSync } from 'node:zlib';

// Download budgets, Brotli 11. The spec sets no size before measuring
// (§21.3); these sit about 10 % above what was measured for 1.0
// (docs/benchmarks/), so that growth is a decision rather than an accident.
const BUDGETS = [
  { name: 'engine (package/pkg/cue_web_bg.wasm)', matches: (p) => p === 'package/pkg/cue_web_bg.wasm', max: 4.6 * 1024 * 1024 },
  // Raised from 35 KiB for the gameplay pass (banners, fine aim, haptics,
  // the players' balls): docs/benchmarks/phase-1-web.md.
  { name: 'control layer (package/*.js)', matches: (p) => /^package\/[^/]+\.js$/.test(p), max: 40 * 1024 },
];

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
const used = BUDGETS.map(() => 0);
for (const path of files.sort()) {
  const data = readFileSync(path);
  const gz = gzipSync(data, { level: 9 }).length;
  const br = brotliCompressSync(data, {
    params: { [constants.BROTLI_PARAM_QUALITY]: 11 },
  }).length;
  const name = relative(root, path).split(sep).join('/');
  console.log(`${name.padEnd(36)}${kib(data.length)}${kib(gz)}${kib(br)}`);
  BUDGETS.forEach((budget, i) => {
    if (budget.matches(name)) used[i] += br;
  });
}

console.log(`\n${'budget, brotli 11'.padEnd(36)}${'used'.padStart(12)}${'limit'.padStart(12)}`);
let over = false;
BUDGETS.forEach((budget, i) => {
  const fits = used[i] <= budget.max;
  over ||= !fits;
  console.log(`${budget.name.padEnd(36)}${kib(used[i])}${kib(budget.max)}  ${fits ? 'ok' : 'OVER BUDGET'}`);
});
if (over && process.env.CUE_SIZE_BUDGET !== 'off') {
  console.error('\nA download budget is exceeded. Shrink the change, or raise the budget in');
  console.error('scripts/size-report.mjs and record why in docs/benchmarks/.');
  process.exit(1);
}
