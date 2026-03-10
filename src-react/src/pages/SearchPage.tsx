import {
  CompositionEventHandler,
  UIEventHandler,
  useEffect,
  useRef,
  useState,
} from "react";
import { toKana } from "wanakana";

import {
  Box,
  Card,
  CloseButton,
  Group,
  Input,
  InputGroup,
  Stack,
  Switch,
} from "@chakra-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { readText } from "@tauri-apps/plugin-clipboard-manager";

import Glossary from "../components/Glossary";

interface Entry {
  term: string;
  reading: string;
  glossary_json: string;
  def_tags: string;
  rules: string;
  score: number;
  sequence: number;
  term_tags: string;
  dict_title: string;
}

function SearchPage() {
  const [q, setQ] = useState("");
  const [entries, setEntries] = useState<Entry[]>([]);
  const [searchTimeout, setSearchTimeout] = useState(0);
  const [furigana, setFurigana] = useState("");
  const [isAutoKana, set_isAutoKana] = useState(true);

  const nSearch = useRef(0);
  const searchboxRef = useRef<HTMLInputElement | null>(null);
  const resultScrollRef = useRef<HTMLElement | null>(null);
  const prevNormQ = useRef("");

  const lang = "ja-JP";

  const [isMonitorClipboard, set_isMonitorclipboard] = useState(false);
  const [, set_clipboardText] = useState("");

  useEffect(() => {
    if (!isMonitorClipboard) return;

    const intervalId = setInterval(async () => {
      const newText = (await readText()).replace(/ /g, "");

      set_clipboardText((clipboardText) => {
        if (clipboardText !== newText) {
          setQ(newText + " ");
          return newText;
        }
        return clipboardText;
      });
    }, 500);

    return () => clearInterval(intervalId);
  }, [isMonitorClipboard]);

  useEffect(() => {
    trySearch();
  }, [q]);

  function trySearch() {
    if (!q.trim()) {
      setEntries([]);
      return;
    }

    if (searchTimeout) {
      clearTimeout(searchTimeout);
    } else if (nSearch.current === 0) {
      // If new, just run immediately. No need for throttling.
      runSearch();
    }

    nSearch.current += 1;

    setSearchTimeout(
      setTimeout(() => {
        if (nSearch.current > 1) runSearch();
      }, 250),
    );
  }

  async function runSearch(isNew = true) {
    let norm_q = q.split(" ")[0].trim();
    const isFromSplit = norm_q !== q.trim();

    if (isAutoKana) {
      norm_q = norm_q.replace(/n$/i, "ん").replace(/[a-z]$/i, "");
    }
    if (!norm_q) return;
    if (isNew && prevNormQ.current === norm_q) return;
    prevNormQ.current = norm_q;

    const ender = /\p{Z}$/u.test(q) ? "" : "*";

    const re = /\<(.+?)\>\[(.+?)\]/g;

    let qTerm = "";
    let qReading = "";

    norm_q.split(re).map((s, i) => {
      switch (i % 3) {
        case 1:
          qTerm += s;
          break;
        case 2:
          qReading += s;
          break;
        default:
          qTerm += s;
          qReading += s;
      }
    });

    qTerm += ender;
    qReading += ender;

    const result = await invoke<Entry[]>("search_yomitan", {
      qTerm,
      qReading,
      limit: 10,
      offset: isNew ? 0 : entries.length,
    });

    if (isNew) {
      if (!result.length && !isFromSplit && /^[^ ]{3,} $/.test(q)) {
        const segs = await invoke<{ surface: string }[]>("tokenize", {
          text: q,
        });
        if (segs.length > 1) {
          const newQ =
            segs.shift()!.surface +
            " " +
            segs.map((s) => s.surface).join("") +
            " ";

          setQ(newQ);
          return;
        }
      }

      if (resultScrollRef.current) {
        resultScrollRef.current.scrollTop = 0;
      }

      setEntries(result);
    } else {
      setEntries([...entries, ...result]);
    }

    return result;
  }

  const onEntriesScrollEnd: UIEventHandler = async ({ target }) => {
    if (!(target instanceof HTMLElement)) return;
    const { scrollHeight, scrollTop } = target;

    if (scrollHeight - 20 > scrollTop + target.getBoundingClientRect().height)
      return;

    const results = await runSearch(false);
    if (!results?.length) {
      target.style.paddingBottom = "50vh";
      return;
    }

    setTimeout(() => {
      target.scrollTop = scrollTop + 100;
    }, 100);
  };

  function onSearchboxChange(q: string) {
    if (isAutoKana) {
      q = toKana(q, {
        useObsoleteKana: true,
        IMEMode: true,
        customKanaMapping: Object.fromEntries(
          "<>[]".split("").map((c) => [c, c]),
        ),
      });
    }

    setQ(q);
  }

  const FURIGANA_REGEX = /^[\p{scx=Hiragana}\p{scx=Katakana}]+$/u;
  const KANJI_REGEX = /([\p{sc=Han}\p{N}々〆ヵヶ]+)/u;

  const updateFurigana: CompositionEventHandler = ({
    data: compositionData,
  }) => {
    if (FURIGANA_REGEX.test(compositionData)) {
      setFurigana(compositionData);
    }
  };

  const addFurigana: CompositionEventHandler = ({
    data: compositionData,
    target,
  }) => {
    const cleanedFuri = [...furigana.replace(/ｎ/g, "ん")]
      .map((c) => katakanaToHiragana(c))
      .join("");

    let parts = compositionData.split(KANJI_REGEX);
    if (parts.length === 1) return;

    const hiraganaParts = parts.map((p) =>
      [...p].map((c) => katakanaToHiragana(c)).join(""),
    );
    const regex = new RegExp(
      `^${hiraganaParts.map((p, idx) => `(${idx & 1 ? ".+" : p})`).join("")}$`,
    );
    let rt: (string | null)[] = furigana.match(regex) || [];
    if (!rt.length) {
      parts = [compositionData];
      rt = [null, cleanedFuri];
    }
    rt.shift();

    const markup = parts
      .map((p, idx) => (idx & 1 ? `<${p}>[${rt[idx]}]` : p))
      .join("");

    setQ((q) => {
      if (!(target instanceof HTMLInputElement)) return markup;

      let { selectionStart } = target;
      selectionStart = selectionStart || 0;
      if (selectionStart < compositionData.length) return markup;

      return (
        q.substring(0, selectionStart - compositionData.length) +
        markup +
        q.substring(selectionStart)
      );
    });
  };

  function katakanaToHiragana(k: string) {
    let c = k.charCodeAt(0);
    return c >= 12449 && c <= 12531
      ? String.fromCharCode(k.charCodeAt(0) - 96)
      : k;
  }

  const clearButton = q.trim() ? (
    <CloseButton
      size={"xs"}
      me={-2}
      onClick={() => {
        setQ("");
        searchboxRef.current ? searchboxRef.current.focus() : null;
      }}
    />
  ) : null;

  return (
    <Stack
      style={{
        margin: "0.5em auto",
        maxWidth: "1000px",
        maxHeight: "100vh",
        display: "flex",
        flexDirection: "column",
        overflowY: "hidden",
      }}
    >
      <form
        lang={lang}
        onSubmit={(e) => {
          e.preventDefault();
        }}
      >
        <Group attached w="full">
          <InputGroup endElement={clearButton}>
            <Input
              id="greet-input"
              type="text"
              ref={searchboxRef}
              value={q}
              onChange={(e) => onSearchboxChange(e.currentTarget.value)}
              onCompositionUpdate={updateFurigana}
              onCompositionEnd={addFurigana}
              autoComplete="off"
              spellCheck={false}
              placeholder="Search..."
            />
          </InputGroup>
          {/* <Button type="submit">Search</Button> */}
        </Group>
        <div style={{ marginTop: "0.5em", display: "flex", gap: "1em" }}>
          <Switch.Root
            checked={isAutoKana}
            onCheckedChange={(d) => set_isAutoKana(d.checked)}
          >
            <Switch.HiddenInput />
            <Switch.Control />
            <Switch.Label>Auto-convert Kana</Switch.Label>
          </Switch.Root>
          <Switch.Root
            checked={isMonitorClipboard}
            onCheckedChange={(d) => set_isMonitorclipboard(d.checked)}
          >
            <Switch.HiddenInput />
            <Switch.Control />
            <Switch.Label>Monitor clipboard</Switch.Label>
          </Switch.Root>
        </div>
      </form>
      <Box
        ref={resultScrollRef}
        as={"ol"}
        listStyleType={"number"}
        style={{ overflowY: "scroll", scrollbarWidth: "none" }}
        onScrollEnd={onEntriesScrollEnd}
      >
        {entries.map(
          ({ term, reading, dict_title, glossary_json, ...it }, i) => (
            <li key={i} lang={lang} style={{ marginLeft: "1.5em" }}>
              <div>
                <Card.Root>
                  <Card.Header>
                    {term}
                    {reading ? ` (${reading})` : ""} 【{dict_title}】
                    {JSON.stringify(it, (_, v) => v || undefined)}
                  </Card.Header>
                  <Card.Body>
                    <div
                      style={{
                        display: "flex",
                        flexDirection: "column",
                        whiteSpace: "pre-wrap",
                      }}
                    >
                      {JSON.parse(glossary_json).map((g: any, i: number) => (
                        <Glossary
                          key={i}
                          glossary={g}
                          onTermClicked={(t) => setQ(t + " ")}
                        />
                      ))}
                    </div>
                  </Card.Body>
                </Card.Root>
              </div>
            </li>
          ),
        )}
      </Box>
    </Stack>
  );
}

export default SearchPage;
