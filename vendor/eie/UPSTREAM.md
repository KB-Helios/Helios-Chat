# Helios Vendor Record

This directory vendors KB01111/EIE for Helios Chat.

- Upstream repository: https://github.com/KB01111/EIE
- Imported commit: `87e690be586f0fde776c9fc1251d56a3cd6356c3`
- Import date: 2026-06-06
- Helios role: default local inference engine managed by the Tauri control plane

The original EIE import included a placeholder `llama.cpp` folder. Helios replaces it with the TurboQuant llama.cpp fork:

- Repository: https://github.com/TheTom/llama-cpp-turboquant
- Imported commit: `7d9715f1f071fa07c7b2ad3dbfd320b314139e65`

Helios builds this tree from the first-run setup wizard and copies the resulting `eie-server` binary into the app-local engine directory.
