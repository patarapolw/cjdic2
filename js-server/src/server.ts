import fastify from "fastify";
import { readdirSync } from "fs";
import path from "path";

import fastifyStatic from "@fastify/static";

const app = fastify({
  logger: true,
});

const YOMITAN_PATH = path.join(__dirname, "../src-tauri/resources/yomitan/ja");

app.register(fastifyStatic, {
  root: YOMITAN_PATH,
  prefix: "/api/files/yomitan/ja",
});

app.get("/api/yomitan/ja/list", async () => {
  return {
    list: readdirSync(YOMITAN_PATH).filter((p) => p.endsWith(".zip")),
  };
});

app.listen({ port: 3000 }).catch((e) => {
  app.log.error(e);
  process.exit(1);
});
