import { defineConfig } from "tsup";
export default defineConfig({
  entry: { index: "src/index.ts", provider: "src/provider.tsx" },
  dts: { entry: ["src/index.ts", "src/provider.tsx"] },
  format: ["esm","cjs"],
  target: "es2018",
  sourcemap: true,
  clean: true
});