cargo build --manifest-path=../Cargo.toml --package secalc_gui_iced --bin secalc_gui_iced --target wasm32-unknown-unknown
wasm-bindgen ..\target\wasm32-unknown-unknown\debug\secalc_gui_iced.wasm --out-dir . --web
