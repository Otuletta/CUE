// Starts the browser for the tests. CI uses the Chrome installed on the
// runner; CHROME_PATH points at another Chromium build instead, such as
// Chrome for Testing.
import { chromium } from 'playwright-core';

/** Launches Chrome headless, with `extra` command-line switches. */
export function launch(extra = []) {
  // Linux runners have no GPU: WebGL 2 runs on SwiftShader, which Chrome
  // only enables on request.
  const gpu = process.platform === 'linux' ? ['--use-angle=swiftshader', '--enable-unsafe-swiftshader'] : [];
  const browser = process.env.CHROME_PATH ? { executablePath: process.env.CHROME_PATH } : { channel: 'chrome' };
  return chromium.launch({ ...browser, args: [...gpu, ...extra] });
}
