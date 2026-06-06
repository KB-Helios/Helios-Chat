/** @type {import('vitest/config').UserConfig} */
module.exports = {
  test: {
    environment: "jsdom",
    setupFiles: ["src/test/setup.ts"],
    globals: true
  }
};
