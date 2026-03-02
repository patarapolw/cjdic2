import fs from "node:fs";

function makeManifestForDir(d) {
  fs.writeFileSync(
    `${d}/manifest.json`,
    JSON.stringify(fs.readdirSync(d).filter((f) => f.endsWith(".zip"))),
  );
}

makeManifestForDir("src-tauri/resources/yomitan/ja");
