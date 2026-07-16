# Build ke WebAssembly

## Yang di-scope
Modul yang di-compile untuk wasm32: `parser`, `preprocessor` (mode in-memory,
tanpa `#include`), `dictionary`, `namespaces`, `legacy_ini`, `v4`, `source_map`,
`utils` (kecuali `utils::version`).

Modul yang DIKECUALIKAN dari build wasm (butuh OS/network, gak relevan di
browser): `language_service` (file watcher + LSP socket), `update_service` &
`sanny_update` (HTTP self-updater, `libloading`), `ide` (parser IDE + winapi),
`sdk` (Windows message hooks). Semua di-gate pakai
`#[cfg(not(target_arch = "wasm32"))]` di `lib.rs`, jadi native build (DLL yang
dipakai Sanny Builder existing) tetap jalan seperti biasa, tidak berubah.

Ditemukan juga satu masalah lama: `src/dictionary/ffi.rs` sebelumnya di-gate
`#![cfg(windows)]` padahal isinya logic murni tanpa API OS apa pun — gate itu
sudah dihapus supaya modul `dictionary` (dipakai `preprocessor`, `v4`, dll)
ikut ke-compile di wasm32 (dan sekaligus jadi cross-platform di Linux/macOS).

## Prasyarat
Sandbox saya cuma punya rustc 1.75 dari apt tanpa std untuk wasm32, jadi saya
TIDAK BISA compile/verify hasil build wasm di sini. Perlu dicek manual di
mesin lokal:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack   # kalau belum ada
```

**Penting:** jangan jalanin `cargo build --release` tanpa `--target`. Tanpa
`--target`, cargo compile untuk host kamu (native) -- dan `sdk`/`ide`/
`sanny_update`/`language_service`/`update_service` itu dari awal memang cuma
untuk Windows (winapi dipakai luas di situ, bukan cuma di `utils::version`),
jadi build native di Linux/Termux/macOS akan tetap gagal di modul-modul itu.
Itu bukan hal baru dari perubahan saya -- proyek ini dari awal cuma nge-target
Windows untuk sisi DLL-nya. Buat wasm, target-nya harus disebut eksplisit
(lihat bagian Build di bawah), biar cargo cuma compile modul yang di-gate
untuk wasm32 dan skip semua yang Windows-only.

## Build
```bash
wasm-pack build --target web --out-dir pkg
# atau tanpa wasm-pack:
cargo build --release --target wasm32-unknown-unknown
```

Kalau pakai `wasm-pack`, hasilnya ada `pkg/core.js` + `pkg/core_bg.wasm` yang
tinggal di-`import` di browser/HTML (persis pola `dbeditor.html`).

## API yang di-expose ke JS (`src/wasm/mod.rs`)
```js
import init, { init_panic_hook, parser_check, WasmNamespaces, WasmOpcodeTable, v4_transform, preprocess_in_memory } from "./pkg/core.js";

await init();
init_panic_hook(); // opsional, biar error message di console lebih jelas

parser_check("0@ = 1"); // -> true/false, valid syntax atau nggak

const ns = new WasmNamespaces();
ns.loadLibrary(await (await fetch("sa.json")).text());

const ini = new WasmOpcodeTable(2); // 2 = SA
ini.loadIni(await (await fetch("SASCM.ini")).text());

v4_transform("~0@", ns, ini); // -> "0B1A: 0@"
preprocess_in_memory(sourceCode); // -> jumlah function yang ke-detect
```

## Yang belum di-handle
- `#include` di preprocessor butuh akses file — di wasm ini akan gagal
  resolve (di-log, gak crash). Kalau perlu, bisa ditambahin virtual
  filesystem (map nama file -> string) dan suntikkan lewat callback JS.
- `parser_check` cuma balikin bool valid/tidak. Kalau butuh AST lengkap ke
  JS, tinggal tambahin `#[derive(serde::Serialize)]` ke semua type di
  `parser/interface.rs` lalu serialize pakai `serde-wasm-bindgen` (sudah
  ditambahin ke Cargo.toml, tinggal dipakai).
- `preprocess_in_memory` baru balikin jumlah function sebagai smoke test;
  gampang diperluas (return list nama function, error list, dll) — API
  publiknya (`Preprocessor.scopes`, dst) sudah `pub`.
