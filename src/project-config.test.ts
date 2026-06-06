/**
 * Tests for project configuration files changed in this PR.
 *
 * Validates the structure and content of:
 * - package.json (renamed to helios-chat, updated scripts/deps)
 * - index.html (updated title and entry point)
 * - src-tauri/capabilities/default.json (Tauri capability definitions)
 * - .gitignore (replaced with project-specific patterns)
 */

import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const ROOT = resolve(__dirname, "..");

// ---------------------------------------------------------------------------
// package.json
// ---------------------------------------------------------------------------

describe("package.json", () => {
  const pkg = JSON.parse(readFileSync(join(ROOT, "package.json"), "utf-8"));

  it("has the correct package name", () => {
    assert.equal(pkg.name, "helios-chat");
  });

  it("is a private package", () => {
    assert.equal(pkg.private, true);
  });

  it("declares ES module type", () => {
    assert.equal(pkg.type, "module");
  });

  it("has a dev script", () => {
    assert.ok(typeof pkg.scripts?.dev === "string", "scripts.dev should be defined");
  });

  it("has a build script that compiles TypeScript before bundling", () => {
    assert.ok(
      typeof pkg.scripts?.build === "string" && pkg.scripts.build.includes("tsc"),
      "scripts.build should include tsc"
    );
  });

  it("has a test script", () => {
    assert.ok(typeof pkg.scripts?.test === "string", "scripts.test should be defined");
  });

  it("has a tauri script", () => {
    assert.ok(typeof pkg.scripts?.tauri === "string", "scripts.tauri should be defined");
  });

  it("declares @tauri-apps/api as a runtime dependency", () => {
    assert.ok(
      "@tauri-apps/api" in (pkg.dependencies ?? {}),
      "@tauri-apps/api should be in dependencies"
    );
  });

  it("declares react as a runtime dependency", () => {
    assert.ok("react" in (pkg.dependencies ?? {}), "react should be in dependencies");
  });

  it("declares react-dom as a runtime dependency", () => {
    assert.ok("react-dom" in (pkg.dependencies ?? {}), "react-dom should be in dependencies");
  });

  it("declares @tauri-apps/cli as a dev dependency", () => {
    assert.ok(
      "@tauri-apps/cli" in (pkg.devDependencies ?? {}),
      "@tauri-apps/cli should be in devDependencies"
    );
  });

  it("declares typescript as a dev dependency", () => {
    assert.ok(
      "typescript" in (pkg.devDependencies ?? {}),
      "typescript should be in devDependencies"
    );
  });

  it("declares vite as a dev dependency", () => {
    assert.ok("vite" in (pkg.devDependencies ?? {}), "vite should be in devDependencies");
  });

  it("does not list bun-specific tooling (migrated away from bun)", () => {
    const allDeps = {
      ...(pkg.dependencies ?? {}),
      ...(pkg.devDependencies ?? {})
    };
    assert.ok(!("bun" in allDeps), "bun should not appear in dependencies");
  });

  it("version is a valid semver string", () => {
    assert.match(
      pkg.version,
      /^\d+\.\d+\.\d+/,
      "version should follow semver format"
    );
  });
});

// ---------------------------------------------------------------------------
// index.html
// ---------------------------------------------------------------------------

describe("index.html", () => {
  const html = readFileSync(join(ROOT, "index.html"), "utf-8");

  it("contains correct page title", () => {
    assert.ok(html.includes("<title>Helios Chat</title>"), "title should be 'Helios Chat'");
  });

  it("contains a root mount point for the React app", () => {
    assert.ok(
      html.includes('<div id="root">'),
      'index.html must contain <div id="root">'
    );
  });

  it("loads main.tsx as a module script", () => {
    assert.ok(
      html.includes('src="/src/main.tsx"'),
      "script src should point to /src/main.tsx"
    );
    assert.ok(
      html.includes('type="module"'),
      "script should have type=module"
    );
  });

  it("declares UTF-8 charset", () => {
    assert.ok(
      html.includes('charset="UTF-8"'),
      'charset should be UTF-8'
    );
  });

  it("has a responsive viewport meta tag", () => {
    assert.ok(
      html.includes('name="viewport"'),
      "viewport meta tag should be present"
    );
  });

  it("sets the html lang attribute to 'en'", () => {
    assert.ok(html.includes('lang="en"'), 'html element should have lang="en"');
  });

  it("does not reference the old Vite + React placeholder title", () => {
    assert.ok(!html.includes("Vite + React"), "old Vite+React title should be absent");
  });
});

// ---------------------------------------------------------------------------
// src-tauri/capabilities/default.json
// ---------------------------------------------------------------------------

describe("src-tauri/capabilities/default.json", () => {
  const capabilitiesPath = join(ROOT, "src-tauri", "capabilities", "default.json");
  const caps = JSON.parse(readFileSync(capabilitiesPath, "utf-8"));

  it("has an identifier field", () => {
    assert.ok(typeof caps.identifier === "string" && caps.identifier.length > 0);
  });

  it("identifier is 'default'", () => {
    assert.equal(caps.identifier, "default");
  });

  it("has a description field", () => {
    assert.ok(typeof caps.description === "string" && caps.description.length > 0);
  });

  it("targets the 'main' window", () => {
    assert.ok(
      Array.isArray(caps.windows) && caps.windows.includes("main"),
      "windows array should include 'main'"
    );
  });

  it("has a non-empty permissions array", () => {
    assert.ok(
      Array.isArray(caps.permissions) && caps.permissions.length > 0,
      "permissions should be a non-empty array"
    );
  });

  it("grants core:default permission", () => {
    assert.ok(
      caps.permissions.includes("core:default"),
      "core:default should be in permissions"
    );
  });

  it("grants dialog:default permission", () => {
    assert.ok(
      caps.permissions.includes("dialog:default"),
      "dialog:default should be in permissions"
    );
  });

  it("grants fs:default permission", () => {
    assert.ok(
      caps.permissions.includes("fs:default"),
      "fs:default should be in permissions"
    );
  });

  it("grants shell:default permission", () => {
    assert.ok(
      caps.permissions.includes("shell:default"),
      "shell:default should be in permissions"
    );
  });

  it("references the desktop schema", () => {
    assert.ok(
      typeof caps.$schema === "string" && caps.$schema.includes("desktop-schema.json"),
      "$schema should reference desktop-schema.json"
    );
  });

  it("does not grant overly broad wildcard permissions", () => {
    for (const perm of caps.permissions) {
      assert.ok(perm !== "*", "wildcard '*' permission must not be granted");
    }
  });
});

// ---------------------------------------------------------------------------
// .gitignore
// ---------------------------------------------------------------------------

describe(".gitignore", () => {
  const gitignore = readFileSync(join(ROOT, ".gitignore"), "utf-8");
  const lines = gitignore.split(/\r?\n/).map((l) => l.trim());

  it("ignores node_modules directory", () => {
    assert.ok(
      lines.some((l) => l === "node_modules/" || l === "node_modules"),
      "node_modules should be ignored"
    );
  });

  it("ignores dist build output", () => {
    assert.ok(
      lines.some((l) => l === "dist/" || l === "dist"),
      "dist should be ignored"
    );
  });

  it("ignores Rust build artifacts in src-tauri/target", () => {
    assert.ok(
      lines.some((l) => l === "src-tauri/target/" || l === "src-tauri/target"),
      "src-tauri/target should be ignored"
    );
  });

  it("ignores codex EIE build directories", () => {
    assert.ok(
      lines.some((l) => l.includes(".codex-eie-")),
      ".codex-eie-*-build/ pattern should be present"
    );
  });

  it("ignores vendor EIE build artifacts", () => {
    assert.ok(
      lines.some((l) => l.includes("vendor/eie/build")),
      "vendor/eie/build* pattern should be present"
    );
  });

  it("ignores llama.cpp build artifacts", () => {
    assert.ok(
      lines.some((l) => l.includes("vendor/eie/llama.cpp/build")),
      "vendor/eie/llama.cpp/build* should be ignored"
    );
  });

  it("ignores log files", () => {
    assert.ok(
      lines.some((l) => l === "*.log"),
      "*.log pattern should be present"
    );
  });

  it("ignores .codex-local directory", () => {
    assert.ok(
      lines.some((l) => l === ".codex-local/" || l === ".codex-local"),
      ".codex-local should be ignored"
    );
  });

  it("does not ignore src directory (source code must be tracked)", () => {
    assert.ok(
      !lines.some((l) => l === "src" || l === "src/"),
      "src directory must not be in .gitignore"
    );
  });

  it("does not have duplicate entries", () => {
    const nonEmpty = lines.filter((l) => l && !l.startsWith("#"));
    const unique = new Set(nonEmpty);
    assert.equal(
      unique.size,
      nonEmpty.length,
      "gitignore should not have duplicate entries"
    );
  });
});
