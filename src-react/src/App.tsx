import "./App.css";

import { CompositionEventHandler, useEffect, useState } from "react";

import { Box, Button, Card, Group, Input, Stack } from "@chakra-ui/react";
import { invoke } from "@tauri-apps/api/core";

import Glossary from "./components/Glossary";
import { Provider } from "./components/ui/provider";

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

function App() {
  const [q, setQ] = useState("");
  const [entries, setEntries] = useState<Entry[]>([]);
  const [searchTimeout, setSearchTimeout] = useState(0);
  const [furigana, setFurigana] = useState("");

  useEffect(() => {
    trySearch();
  }, [q]);

  async function trySearch() {
    if (!q.trim()) {
      setEntries([]);
      return;
    }

    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }

    setSearchTimeout(
      setTimeout(() => {
        runSearch();
      }, 250),
    );
  }

  async function runSearch() {
    const ender = /\p{Z}$/u.test(q) ? "" : "*";

    const re = /\<(.+?)\>\[(.+?)\]/g;

    let qTerm = "";
    let qReading = "";

    q.trim()
      .split(re)
      .map((s, i) => {
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

    setEntries(
      await invoke("search_yomitan", {
        qTerm,
        qReading,
        limit: 10,
        offset: 0,
      }),
    );
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

  const addFurigana: CompositionEventHandler = ({ data: compositionData }) => {
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
    setQ(markup);
  };

  function katakanaToHiragana(k: string) {
    let c = k.charCodeAt(0);
    return c >= 12449 && c <= 12531
      ? String.fromCharCode(k.charCodeAt(0) - 96)
      : k;
  }

  return (
    <Provider>
      <Stack maxW={"1000px"} margin={"0.5em auto"}>
        <form
          onSubmit={(e) => {
            e.preventDefault();
          }}
        >
          <Group attached w="full">
            <Input
              id="greet-input"
              value={q}
              onChange={(e) => setQ(e.currentTarget.value)}
              onCompositionUpdate={updateFurigana}
              onCompositionEnd={addFurigana}
              autoComplete="off"
              spellCheck={false}
              placeholder="Search..."
            />
            <Button type="submit">Search</Button>
          </Group>
        </form>
        <Box as={"ol"} listStyleType={"number"} style={{ margin: "1em" }}>
          {entries.map(
            ({ term, reading, dict_title, glossary_json, ...it }, i) => (
              <li key={i}>
                <details>
                  <summary>
                    {term} ({reading}) 【{dict_title}】
                  </summary>

                  <Card.Root>
                    <Card.Header>
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
                        {JSON.parse(glossary_json).map((g: any) => (
                          <Glossary
                            glossary={g}
                            onTermClicked={(t) => setQ(t + " ")}
                          />
                        ))}
                      </div>
                    </Card.Body>
                  </Card.Root>
                </details>
              </li>
            ),
          )}
        </Box>
      </Stack>
    </Provider>
  );
}

export default App;
