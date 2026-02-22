import "./App.css";

import { useEffect, useState } from "react";

import { Button, Group, Input, Stack } from "@chakra-ui/react";
import { invoke } from "@tauri-apps/api/core";

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
              placeholder="Search..."
            />
            <Button type="submit">Search</Button>
          </Group>
        </form>
        <ol>
          {entries.map((it, i) => (
            <li key={i}>{JSON.stringify(it)}</li>
          ))}
        </ol>
      </Stack>
    </Provider>
  );
}

export default App;
