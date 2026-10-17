// Writes package/manifest.json: the files a host needs, with sizes and
// SHA-256 digests. The wrapper reads the uncompressed WASM size from it to
// show real loading progress (spec §18.2, §20.6).
// Usage: node scripts/manifest.mjs web/dist/package <version>
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const [dir = 'web/dist/package', version = '0.0.0'] = process.argv.slice(2);
const describe = (file) => {
  const data = readFileSync(join(dir, file));
  return { file: file.replace(/^pkg\//, ''), bytes: data.length, sha256: createHash('sha256').update(data).digest('hex') };
};

const manifest = {
  name: 'cue',
  version,
  wasm: describe('pkg/cue_web_bg.wasm'),
  glue: describe('pkg/cue_web.js'),
};
writeFileSync(join(dir, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`manifest: ${manifest.wasm.bytes} wasm bytes, sha256 ${manifest.wasm.sha256.slice(0, 12)}…`);
