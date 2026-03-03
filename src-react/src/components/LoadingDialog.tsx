import { useEffect, useRef, useState } from "react";

import { Dialog, Portal } from "@chakra-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  BaseDirectory,
  exists,
  readDir,
  writeFile,
} from "@tauri-apps/plugin-fs";

import { supabase } from "../lib/supabaseClient";

interface LoadYomitanZipDirResult {
  new_dicts: string[];
  to_be_removed_dicts: string[];
}

interface YomitanZipImportProgress {
  message: string;
  current: number;
  total: number;
  steps: number;
}

interface Progress {
  message: string;
  current?: number;
  total?: number;
}

function LoadingDialog() {
  const [messages, setMessages] = useState<Progress[]>([]);

  const isInit = useRef(false);

  useEffect(() => {
    if (isInit.current) return;
    isInit.current = true;

    invoke("init_yomitan").then(async () => {
      const { data, error } = await supabase.storage.from("yomitan").list("ja");
      if (error) throw error;

      for (const d of data || []) {
        const filepath = `yomitan/ja/${d.name}`;
        if (
          await exists(filepath, {
            baseDir: BaseDirectory.AppData,
          })
        )
          continue;

        const {
          data: { publicUrl },
        } = supabase.storage.from("yomitan").getPublicUrl(`ja/${d.name}`);

        // await downloadURL(publicUrl, filepath, (received, total) => {});
      }

      setMessages([]);
    });

    listen<LoadYomitanZipDirResult>("load-yomitan-dir", ({ payload }) => {
      setMessages((messages) => {
        if (payload.new_dicts.length > 0) {
          messages = [
            ...messages,
            { message: "Importing Yomitan zip from the app bundle:" },
            ...payload.new_dicts.map((d) => ({ message: `- ${d}` })),
          ];
        }

        if (payload.to_be_removed_dicts.length > 0) {
          messages = [
            ...messages,
            { message: "Removing old Yomitan zip:" },
            ...payload.to_be_removed_dicts.map((d) => ({ message: `- ${d}` })),
          ];
        }

        return messages;
      });
    });

    listen<YomitanZipImportProgress>(
      "yomitan-import-progress",
      ({ payload }) => {
        setMessages((messages) => {
          const prev = messages[messages.length - 1];
          if (prev?.message === payload.message) {
            messages = messages.slice(0, messages.length - 1);
          }

          messages = [...messages, payload];

          return messages;
        });
      },
    );
  }, []);

  return (
    <Dialog.Root open={messages.length > 0}>
      <Dialog.Trigger />
      <Portal>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content>
            <Dialog.CloseTrigger />
            <Dialog.Header>
              <Dialog.Title />
            </Dialog.Header>
            <Dialog.Body>
              {messages.length
                ? messages.map((m, i) => (
                    <p key={i}>
                      {m.total
                        ? `${m.message} (${m.current}/${m.total})`
                        : m.message}
                    </p>
                  ))
                : "Loading..."}
            </Dialog.Body>
            <Dialog.Footer />
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  );
}

export default LoadingDialog;

async function downloadURL(
  url: string,
  filepath: string,
  progressCallback: (received: number, total: number) => void,
) {
  const r = await fetch(url);
  if (!r.ok) throw r;
  if (!r.body) throw r;

  const contentLength = r.headers.get("Content-Length");
  const total = contentLength ? parseInt(contentLength) : 0;

  let receivedBytes = 0;

  const reader = r.body.getReader();
  const stream = new ReadableStream({
    start(controller) {
      function push() {
        reader
          .read()
          .then(({ done, value }) => {
            if (done) {
              controller.close();
              return;
            }
            receivedBytes += value.length;

            // Calculate and log progress
            if (total) progressCallback(receivedBytes, total);

            controller.enqueue(value);
            push(); // Read the next chunk
          })
          .catch((error) => {
            console.error("Stream reading error:", error);
            controller.error(error);
          });
      }

      push(); // Start reading the data
    },
  });

  await writeFile(filepath, stream, { baseDir: BaseDirectory.AppData });
}
