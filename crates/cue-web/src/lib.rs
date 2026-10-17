//! # cue-web
//!
//! The boundary between CUE and a host web page. The JavaScript wrapper in
//! `web/package` calls into this crate; nothing here may assume anything about
//! the host's router, styles or framework.
//!
//! Phase 0 exposes only the build version. Mounting, lifecycle and events are
//! added in Phase 1.

use wasm_bindgen::prelude::*;

/// Version of the CUE build, for diagnostics and the `ready` event.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}
