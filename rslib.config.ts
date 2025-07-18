import { defineConfig } from "@rslib/core";
import { pluginDts } from "rsbuild-plugin-dts";

export default defineConfig({
  lib: [
    { format: "esm", syntax: "es2021" },
    { format: "cjs", syntax: "es2021" },
  ],
  output: {
    target: "node",
  },
  plugins: [pluginDts()],
});
