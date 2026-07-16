#[macro_use]
pub mod common_ffi;

// --- cross-platform core (compiles for native AND wasm32) ---
pub mod dictionary;
pub mod legacy_ini;
pub mod namespaces;
pub mod parser;
pub mod preprocessor;
pub mod source_map;
pub mod utils;
pub mod v4;

#[macro_use]
extern crate lazy_static;

// --- native-only (desktop DLL): relies on Windows APIs, file watching,
// networking, or dynamic library loading that don't exist in a wasm/browser
// context ---
#[cfg(not(target_arch = "wasm32"))]
pub mod ide;
#[cfg(not(target_arch = "wasm32"))]
pub mod language_service;
#[cfg(not(target_arch = "wasm32"))]
pub mod sanny_update;
#[cfg(not(target_arch = "wasm32"))]
pub mod sdk;
#[cfg(not(target_arch = "wasm32"))]
pub mod update_service;

// --- wasm-bindgen bindings exposed to JS ---
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
mod native_init {
    use ctor::ctor;
    use simplelog::*;

    #[ctor]
    fn main() {
        let config = ConfigBuilder::new()
            .set_level_padding(LevelPadding::Off)
            .set_time_to_local(true)
            .set_thread_level(LevelFilter::Off)
            .build();

        let cwd = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let _ = WriteLogger::init(
            LevelFilter::max(),
            config,
            std::fs::File::create(cwd.join("core.log")).unwrap(),
        );

        log::info!("core library {} loaded", env!("CARGO_PKG_VERSION"));
    }
}
