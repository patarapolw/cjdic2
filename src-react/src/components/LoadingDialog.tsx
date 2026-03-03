import { useEffect, useRef, useState } from "react";

import { Dialog, Portal } from "@chakra-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { BaseDirectory, exists } from "@tauri-apps/plugin-fs";
import { fetch } from "@tauri-apps/plugin-http";

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
      const new_dicts = new Set<string>();
      // const to_be_removed_dicts = new Set<string>();

      if (lang === "ja") {
        await getPixiv().catch((e) => {
          console.error(e);
        });
        async function getPixiv() {
          const filename = "Pixiv.zip";
          new_dicts.add(filename);

          const filepath = `yomitan/${lang}/${filename}`;
          if (
            !(await exists(filepath, {
              baseDir: BaseDirectory.AppData,
            }))
          ) {
            const publicUrl = await ghLatestReleaseURL(
              "MarvNC/pixiv-yomitan",
              /^Pixiv_.+\.zip$/,
            );

            if (publicUrl) {
              await invoke("download_url", { url: publicUrl, filepath });
            }
          }
        }
      }

      let isSupabase = true;
      await getSupabase().catch((e) => {
        console.error(e);
        isSupabase = false;
      });
      async function getSupabase() {
        const { data, error } = await supabase.storage
          .from("yomitan")
          .list(lang);
        if (error) throw error;

        for (const d of data || []) {
          new_dicts.add(d.name);
          const filepath = `yomitan/${lang}/${d.name}`;
          if (
            !(await exists(filepath, {
              baseDir: BaseDirectory.AppData,
            }))
          ) {
            const {
              data: { publicUrl },
            } = supabase.storage
              .from("yomitan")
              .getPublicUrl(`${lang}/${d.name}`);

            await invoke("download_url", { url: publicUrl, filepath });
          }
        }
      }

      if (isSupabase) {
        // console.log([...new_dicts], [...to_be_removed_dicts]);
        // for (const f of await readDir(`yomitan/${lang}`, {
        //   baseDir: BaseDirectory.AppData,
        // })) {
        //   if (f.isFile && f.name.endsWith(".zip")) {
        //     if (new_dicts.has(f.name)) {
        //       new_dicts.delete(f.name);
        //     } else {
        //       to_be_removed_dicts.add(f.name);
        //     }
        //   }
        // }
        // for (const bundleName of Array.from(new_dicts).sort()) {
        //   updateProgress({
        //     message: `Importing ${bundleName}`,
        //   });
        //   await invoke("import_yomitan_dict", {
        //     bundleName,
        //     lang,
        //   });
        // }
        // for (const bundleName of Array.from(to_be_removed_dicts).sort()) {
        //   updateProgress({
        //     message: `Removing old ${bundleName}`,
        //   });
        //   await invoke("remove_yomitan_dict", {
        //     bundleName,
        //     lang,
        //   });
        // }
      }

      setMessages([]);
    }

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
