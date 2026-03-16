import {
  CompositionEventHandler,
  UIEventHandler,
  useEffect,
  useRef,
  useState,
} from "react";
import { useSearchParams } from "react-router";
import { toHiragana, toKana } from "wanakana";

import {
  Box,
  Card,
  CloseButton,
  Field,
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
  const [searchParams, set_searchParams] = useSearchParams();

  useEffect(() => {
    let q = searchParams.get("q") || "";
    if (q) set_q(q);
  }, [searchParams]);

  function searchParams_setQ(q: string) {
    if (searchParams.get("q") === q) return;
    set_searchParams({ q });
  }

  const [q, set_q] = useState("");
  const [entries, set_entries] = useState<Entry[]>([]);
  const [searchTimeout, set_searchTimeout] = useState(0);
  const [furigana, set_furigana] = useState("");
  const [isAutoKana, set_isAutoKana] = useState(true);
  const [isScrollEnd, set_isScrollEnd] = useState(false);

  const nSearch = useRef(0);
  const searchboxRef = useRef<HTMLInputElement | null>(null);
  const resultScrollRef = useRef<HTMLElement | null>(null);

  const lang = "ja-JP";

  const [isMonitorClipboard, set_isMonitorclipboard] = useState(false);
  const [, set_clipboardText] = useState("");

  useEffect(() => {
    if (!isMonitorClipboard) return;

    const intervalId = setInterval(async () => {
      const newText = (await readText()).replace(/ /g, "");

      set_clipboardText((clipboardText) => {
        if (clipboardText !== newText) {
          searchParams_setQ(newText + " ");
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
      set_entries([]);
      return;
    }

    if (searchTimeout) {
      clearTimeout(searchTimeout);
    } else if (nSearch.current === 0) {
      // If new, just run immediately. No need for throttling.
      runSearch();
    }

    nSearch.current += 1;

    set_searchTimeout(
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

    norm_q = norm_q.replace(/＊/g, "*").replace(/？/g, "?").replace(/^〜/, "~");

    if (!norm_q) return;

    const ender =
      /\p{Z}$/u.test(q) || [..."*?[]"].some((c) => q.includes(c)) ? "" : "*";

    const re = /\<(.+?)\>{(.+?)}/g;

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
      const [s1, prefix = "", s2 = ""] = q.split(/^([\*\?\[\]\~]*)/);
      const text = s1 || s2;

      if (!result.length && !isFromSplit && /^[^ ]{3,} $/.test(text)) {
        const segs = await invoke<{ surface: string }[]>("tokenize", {
          text,
        });
        if (segs.length > 1) {
          const newQ =
            segs.shift()!.surface +
            " " +
            segs.map((s) => s.surface).join("") +
            " ";

          searchParams_setQ(prefix + newQ);
          return;
        }
      }

      if (resultScrollRef.current) {
        resultScrollRef.current.scrollTop = 0;
        set_isScrollEnd(false);
      }

      set_entries(result);
    } else {
      set_entries([...entries, ...result]);
    }

    return result;
  }

  const onEntriesScrollEnd: UIEventHandler = async ({ target }) => {
    if (!(target instanceof HTMLElement)) return;
    const { scrollHeight, scrollTop } = target;

    if (
      isScrollEnd ||
      scrollHeight - window.innerHeight / 2 >
        scrollTop + target.getBoundingClientRect().height
    )
      return;

    const results = await runSearch(false);
    if (!results?.length) {
      target.style.paddingBottom = "50vh";
      set_isScrollEnd(true);
      return;
    }
  };

  function onSearchboxChange(q: string) {
    if (isAutoKana) {
      q = toKana(q, {
        useObsoleteKana: true,
        IMEMode: true,
        customKanaMapping: Object.fromEntries([..."<>{}"].map((c) => [c, c])),
      });
    }

    set_q(q);
  }

  const FURIGANA_REGEX = /^[\p{scx=Hiragana}\p{scx=Katakana}]+$/u;
  const KANJI_REGEX = /([\p{sc=Han}\p{N}々〆ヵヶ]+)/u;

  const updateFurigana: CompositionEventHandler = ({
    data: compositionData,
  }) => {
    if (FURIGANA_REGEX.test(compositionData)) {
      set_furigana(compositionData);
    }
  };

  const addFurigana: CompositionEventHandler = ({
    data: compositionData,
    target,
  }) => {
    const cleanedFuri = [...furigana.replace(/ｎ/g, "ん")]
      .map((c) => toHiragana(c, { passRomaji: true }))
      .join("");

    let parts = compositionData.split(KANJI_REGEX);
    if (parts.length === 1) return;

    const hiraganaParts = parts.map((p) =>
      [...p].map((c) => toHiragana(c, { passRomaji: true })).join(""),
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
      .map((p, idx) => (idx & 1 ? `<${p}>{${rt[idx]}}` : p))
      .join("");

    set_q((q) => {
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

  const clearButton = q.trim() ? (
    <CloseButton
      size={"xs"}
      me={-2}
      onClick={() => {
        searchParams_setQ(q);
        searchParams_setQ("");
        set_q("");
        searchboxRef.current ? searchboxRef.current.focus() : null;
      }}
    />
  ) : null;

  function onTermClicked(t: string) {
    searchParams_setQ(q);
    searchParams_setQ(t + " ");
  }

  function JSONdumpClean(o: Record<string, any>) {
    const p = Object.entries(o).filter(([, v]) => v || v === 0);
    return p.length ? JSON.stringify(Object.fromEntries(p), null, 1) : null;
  }

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
        <Field.Root w="full">
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
          {q.includes(" ") ? (
            <Field.HelperText>
              Space (" ") to mark word boundary for search
            </Field.HelperText>
          ) : null}
          {[..."*?[]"].some((c) => q.includes(c)) ? (
            <Field.HelperText>
              <code>{"*?[]"}</code> can be used as
              <a
                href="https://en.wikipedia.org/wiki/Glob_(programming)#Syntax"
                target="_blank"
                rel="noopener noreferrer"
              >
                glob
              </a>
            </Field.HelperText>
          ) : null}
        </Field.Root>
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
                  <Card.Header display={"block"}>
                    <a
                      onClick={() =>
                        onTermClicked(
                          reading && reading !== term
                            ? `<${term}>{${reading}}`
                            : term,
                        )
                      }
                    >
                      {term}
                    </a>
                    {reading && reading !== term ? (
                      <>
                        {"（"}
                        <a onClick={() => onTermClicked(reading)}>{reading}</a>
                        {"）"}
                      </>
                    ) : null}
                    <button onClick={() => onTermClicked("~" + term)}>
                      🔗
                    </button>
                    <span>【{dict_title}】</span>
                    <span>{JSONdumpClean(it)}</span>
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
                          onTermClicked={onTermClicked}
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
