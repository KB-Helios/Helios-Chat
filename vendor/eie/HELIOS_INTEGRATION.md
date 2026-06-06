# Helios EIE Integration

Helios treats EIE as a managed sidecar runtime. The desktop app performs prerequisite detection, runs the CMake build, writes `eie.engine.yaml`, starts/stops the EIE process, and communicates with EIE through its OpenAI-compatible HTTP API.

## Runtime Contract

Helios expects these endpoints:

- `GET /health`
- `GET /v1/models`
- `POST /v1/chat/completions`
- `GET /v1/admin/models/discover`
- `POST /v1/admin/models/load`
- `POST /v1/admin/models/unload`
- `GET /v1/admin/vram/status`
- `GET /metrics`

The upstream EIE import already defines the route surface in `server/api.cpp`. The llama.cpp TurboQuant source is vendored under `llama.cpp` so CMake can link real inference targets instead of the original placeholder target.

## Build From Helios

The Tauri command `setup_build_eie` runs:

```powershell
cmake -S vendor/eie -B <app-data>/engine/build-cuda -DGGML_CUDA=ON -DCMAKE_BUILD_TYPE=Release
cmake --build <app-data>/engine/build-cuda --config Release
```

If CUDA is unavailable but Git, CMake, and MSVC C++ tools are present, Helios uses a CPU build directory instead:

```powershell
cmake -S vendor/eie -B <app-data>/engine/build-cpu -DCMAKE_BUILD_TYPE=Release
cmake --build <app-data>/engine/build-cpu --config Release
```

Build output is logged to `<app-data>/logs/eie-build.log`; the selected `eie-server.exe` is copied to `<app-data>/engine/eie-server.exe`.
