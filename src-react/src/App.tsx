import "./App.css";

import { useEffect, useState } from "react";

import { Box, Button, Card, Group, Input, Stack } from "@chakra-ui/react";
import { invoke } from "@tauri-apps/api/core";

import Glossary from "./components/Glossary";
import { Provider } from "./components/ui/provider";

function App() {
  const [q, setQ] = useState("");
  const [entries, setEntries] = useState<any[]>([]);

  useEffect(() => {
    doSearch();
  }, [q]);

  async function doSearch() {
    if (!q.trim()) {
      setEntries([]);
      return;
    }

    const ender = q.endsWith(" ") ? "" : "*";

    const qTerm = q.trim() + ender;
    const qReading = q.trim() + ender;

    setEntries(
      await invoke("search_yomitan", {
        qTerm,
        qReading,
        limit: 10,
        offset: 0,
      }),
    );
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
              autoComplete="off"
              spellCheck={false}
              placeholder="Search..."
            />
            <Button type="submit">Search</Button>
          </Group>
        </form>
        <Box as={"ol"} listStyleType={"number"} style={{ margin: "1em" }}>
          {entries.map(({ term, reading, glossary_json, ...it }, i) => (
            <li key={i}>
              <details>
                <summary>
                  {term} ({reading})
                </summary>

                <Card.Root>
                  <Card.Header>{JSON.stringify(it)}</Card.Header>
                  <Card.Body>
                    {JSON.parse(glossary_json).map((g: any) => (
                      <Glossary
                        glossary={g}
                        onTermClicked={(t) => setQ(t + " ")}
                      />
                    ))}
                  </Card.Body>
                </Card.Root>
              </details>
            </li>
          ))}
        </Box>
      </Stack>
    </Provider>
  );
}

export default App;
