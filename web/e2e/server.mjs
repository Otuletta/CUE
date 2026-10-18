// A static server for the browser tests: web/dist at `/`, and this folder's
// host page at `/e2e/`. Every response carries the strict Content Security
// Policy from docs/web-integration.md, so a violation fails the tests.
import { createReadStream } from 'node:fs';
import { stat } from 'node:fs/promises';
import { createServer } from 'node:http';
import { dirname, extname, join, normalize, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const dist = join(here, '..', 'dist');

export const CSP = [
  "default-src 'self'",
  "script-src 'self' 'wasm-unsafe-eval'",
  "style-src 'self'",
  "img-src 'self'",
  "connect-src 'self'",
  "object-src 'none'",
  "base-uri 'none'",
].join('; ');

const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json',
  '.wasm': 'application/wasm',
};

/** Maps a URL path into `root`, refusing anything that climbs out of it. */
function inside(root, path) {
  const file = normalize(join(root, decodeURIComponent(path)));
  return file === root || file.startsWith(root + sep) ? file : null;
}

/** Starts the server on a free port; resolves with its origin and a `close`. */
export async function serve() {
  const server = createServer(async (req, res) => {
    const { pathname } = new URL(req.url, 'http://localhost');
    const file = pathname.startsWith('/e2e/')
      ? inside(here, pathname.slice('/e2e/'.length))
      : inside(dist, pathname.endsWith('/') ? `${pathname}index.html` : pathname);
    const info = file && (await stat(file).catch(() => null));
    if (!info?.isFile()) {
      res.writeHead(404, { 'Content-Security-Policy': CSP }).end();
      return;
    }
    res.writeHead(200, {
      'Content-Type': TYPES[extname(file)] ?? 'application/octet-stream',
      'Content-Length': info.size,
      'Content-Security-Policy': CSP,
      'Cache-Control': 'no-cache',
    });
    createReadStream(file).pipe(res);
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address();
  return {
    origin: `http://127.0.0.1:${port}`,
    close: () => new Promise((resolve) => server.close(resolve)),
  };
}
