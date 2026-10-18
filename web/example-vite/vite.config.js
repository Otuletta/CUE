// The only CUE-specific setting: keep the package out of the dev server's
// dependency pre-bundling, which would move the module away from its
// WebAssembly. Production builds need nothing.
export default {
  base: './',
  optimizeDeps: { exclude: ['@tuletta/cue'] },
};
