import { useEffect, useRef, useState } from "react";

import { Dialog, Portal } from "@chakra-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

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

    invoke("init_yomitan").then(() => {
      setMessages([]);
    });

    listen<LoadYomitanZipDirResult>("load-yomitan-dir", ({ payload }) => {
      setMessages((messages) => {
        if (payload.new_dicts.length > 0) {
          messages = [
            ...messages,
            { message: "Importing Yomitan zip in resources/ folder:" },
            ...payload.new_dicts.map((d) => ({ message: `- ${d}` })),
          ];
        }

        if (payload.to_be_removed_dicts.length > 0) {
          messages = [
            ...messages,
            { message: "Removing Yomitan zip:" },
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
