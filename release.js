import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

spawnSync("pnpm", ["run", "build:desktop"], {
  stdio: "inherit",
  shell: true,
});
spawnSync("pnpm", ["run", "build:android"], {
  stdio: "inherit",
  shell: true,
});

const file_stem = fs
  .readdirSync("target/release/bundle/msi", { withFileTypes: true })
  .sort((f) => {
    return fs.statSync(path.join(f.parentPath, f.name)).mtime;
  })
  .pop()
  .name.split("_x64")[0];

const apk_dir = "target/release/bundle/apk";
try {
  fs.mkdirSync(apk_dir);
} catch (e) {}

fs.copyFileSync(
  "src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk",
  path.join(apk_dir, `${file_stem}.apk`),
);
