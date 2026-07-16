//! wasm-bindgen bindings for the browser build.
//!
//! Scope: only the pure "compiler front-end" logic is exposed here --
//! parsing, preprocessing (in-memory, no `#include` resolution), the
//! GXT/legacy opcode table, namespaces (SBL command libraries) and the v4
//! syntax transform. Anything that depends on the filesystem, sockets,
//! dynamic-library loading or Windows APIs (language server, auto-updater,
//! IDE file watcher) is intentionally left out of the wasm target -- see
//! `lib.rs` for the cfg gating.
//!
//! All state (Namespaces / OpcodeTable / Dict) is created fresh from strings
//! passed in from JS, since there is no real filesystem to load from in the
//! browser.

use wasm_bindgen::prelude::*;

use crate::dictionary::dictionary_str_by_str::DictStrByStr;
use crate::legacy_ini::{Game, OpcodeTable};
use crate::namespaces::namespaces::Namespaces;
use crate::preprocessor::PreProcessorBuilder;

/// Call once from JS before anything else, to get readable panic messages
/// in the browser console instead of an opaque "unreachable executed".
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Parses a single script expression/statement and reports whether it's
/// syntactically valid. (Full AST -> JS serialization can be added later by
/// deriving `serde::Serialize` on the types in `parser::interface`.)
#[wasm_bindgen]
pub fn parser_check(input: &str) -> bool {
    crate::parser::parse(input).is_ok()
}

/// Namespaces: a loaded SBL command library (e.g. `sa.json`), used to
/// resolve command names/params for the v4 transform.
#[wasm_bindgen]
pub struct WasmNamespaces(pub(crate) Namespaces);

#[wasm_bindgen]
impl WasmNamespaces {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmNamespaces {
        WasmNamespaces(Namespaces::new())
    }

    /// Load a library from its JSON content (fetch the .json in JS and pass
    /// the text in here -- there's no filesystem access in wasm).
    #[wasm_bindgen(js_name = loadLibrary)]
    pub fn load_library(&mut self, json_content: &str) -> bool {
        self.0.load_library_from_str(json_content).is_some()
    }
}

impl Default for WasmNamespaces {
    fn default() -> Self {
        Self::new()
    }
}

/// The legacy opcode table (SASCM.ini-style), keyed by game.
/// game: 0=GTA3, 1=VC, 2=SA, 3=LCS, 4=VCS, 5=SAMOBILE
#[wasm_bindgen]
pub struct WasmOpcodeTable(pub(crate) OpcodeTable);

#[wasm_bindgen]
impl WasmOpcodeTable {
    #[wasm_bindgen(constructor)]
    pub fn new(game: u8) -> WasmOpcodeTable {
        WasmOpcodeTable(OpcodeTable::new(Game::from(game)))
    }

    /// Feed the content of an .ini opcode file line by line.
    #[wasm_bindgen(js_name = loadIni)]
    pub fn load_ini(&mut self, ini_content: &str) {
        for line in ini_content.lines() {
            self.0.parse_line(line);
        }
    }
}

/// Runs the v4 syntax transform (e.g. `~0@` -> `0B1A: 0@`) on a single line.
/// Returns `undefined` (JS) / `None` if the line couldn't be transformed.
#[wasm_bindgen]
pub fn v4_transform(input: &str, ns: &WasmNamespaces, legacy_ini: &WasmOpcodeTable) -> Option<String> {
    let const_lookup = DictStrByStr::default();
    crate::v4::transform(input, &ns.0, &legacy_ini.0, &const_lookup)
}

/// Preprocesses a whole script source held in memory (no `#include`
/// resolution, since there's no filesystem in the browser -- includes will
/// simply fail to resolve and get logged).
/// Returns the number of scoped functions found, as a minimal smoke-test
/// output; extend `Preprocessor`'s public API as needed for richer results.
#[wasm_bindgen]
pub fn preprocess_in_memory(source: &str) -> usize {
    let mut pp = PreProcessorBuilder::new().build();
    let _ = pp.parse_in_memory(source);
    pp.scopes.functions.len()
}
