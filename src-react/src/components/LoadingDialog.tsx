import { useEffect, useRef, useState } from "react";

import { Dialog, Portal } from "@chakra-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { supabase } from "../lib/supabaseClient";

interface YomitanDictEntry {
  url: string;
  filepath: string;
}

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

interface DownloadProgress {
  url: string;
  filepath: string;
  content_length: number;
  downloaded: number;
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

    downloadAssets();
    async function downloadAssets(lang = "ja") {
      const dicts: YomitanDictEntry[] = [];

      if (lang === "ja") {
        {
          const filename = "Pixiv.zip";
          const filepath = `yomitan/${lang}/${filename}`;

          dicts.push({
            filepath,
            url:
              (await ghLatestReleaseURL(
                "MarvNC/pixiv-yomitan",
                /^Pixiv_.+\.zip$/,
              ).catch((e) => {
                console.error(e);
                return "";
              })) || "",
          });
        }

        {
          const { data, error } = await supabase.storage
            .from("yomitan")
            .list(lang);
          if (error) throw error;

          for (const d of data || []) {
            const filepath = `yomitan/${lang}/${d.name}`;

            dicts.push({
              filepath,
              url: supabase.storage
                .from("yomitan")
                .getPublicUrl(`${lang}/${d.name}`).data.publicUrl,
            });
          }
        }
      }

      await invoke("init_yomitan", { dicts, lang });
      setMessages([]);

      // empty query warmup
      await invoke("search_yomitan", {
        qTerm: "",
        qReading: "",
        limit: 1,
        offset: 0,
      });
    }

    listen<LoadYomitanZipDirResult>("load-yomitan-dir", ({ payload }) => {
      setMessages((messages) => {
        if (payload.new_dicts.length > 0) {
          messages = [
            ...messages,
            { message: "Importing Yomitan zip:" },
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
        updateProgress(payload);
      },
    );

    listen<DownloadProgress>("download-url-progress", ({ payload }) => {
      updateProgress({
        message: `Downloading ${payload.filepath} (MiB)`,
        current: payload.downloaded >> 20,
        total: payload.content_length >> 20,
      });
    });
  }, []);

  function updateProgress(payload: Progress) {
    setMessages((messages) => {
      const prev = messages[messages.length - 1];
      if (prev?.message === payload.message) {
        messages = messages.slice(0, messages.length - 1);
      }

      messages = [...messages, payload];

      return messages;
    });
  }

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

/**
 * @see https://docs.github.com/en/rest/releases/releases?apiVersion=2022-11-28#get-the-latest-release
 */
async function ghLatestReleaseURL(user_repo: string, assetName: RegExp) {
  const releaseData: {
    assets: { name: string; browser_download_url: string }[];
  } = await fetch(`https://api.github.com/repos/${user_repo}/releases/latest`, {
    headers: {
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
      "User-Agent": "cjdic2",
    },
  }).then((r) => r.json());

  const a = releaseData.assets.find((a) => assetName.test(a.name));
  if (!a) return;
  return a.browser_download_url;
}
