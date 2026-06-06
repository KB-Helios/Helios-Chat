# Third-Party Notices

Helios Chat vendors local inference components to make EIE the default desktop engine.

## EIE

- Source: https://github.com/KB01111/EIE
- Vendored commit: `87e690be586f0fde776c9fc1251d56a3cd6356c3`
- License: Apache License 2.0
- Location: `vendor/eie`

## llama.cpp TurboQuant Fork

- Source: https://github.com/TheTom/llama-cpp-turboquant
- Vendored commit: `7d9715f1f071fa07c7b2ad3dbfd320b314139e65`
- License: see `vendor/eie/llama.cpp/LICENSE` and `vendor/eie/llama.cpp/licenses`
- Location: `vendor/eie/llama.cpp`

## React, Tauri, Rust, and JavaScript Dependencies

Project dependencies are declared in `package.json` and `src-tauri/Cargo.toml`; their license metadata should be audited during release packaging.
