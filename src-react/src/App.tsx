import "./App.css";

import { useEffect, useState } from "react";

import { Button, Group, Input, Stack } from "@chakra-ui/react";
import { invoke } from "@tauri-apps/api/core";

import { Provider } from "./components/ui/provider";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  useEffect(() => {
    invoke("list_entries").then((r) => {
      setGreetMsg(JSON.stringify(r));
    });
  }, [name]);

  async function greet() {
    await invoke("add_entry", { name });
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

    setName("");
  }

  return (
    <Provider>
      <Stack maxW={"1000px"} margin={"0.5em auto"}>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            greet();
          }}
        >
          <Group attached w="full">
            <Input
              id="greet-input"
              value={name}
              onChange={(e) => setName(e.currentTarget.value)}
              autoComplete="off"
              placeholder="Enter a name..."
            />
            <Button type="submit">Search</Button>
          </Group>
        </form>
        <p>{greetMsg}</p>
      </Stack>
    </Provider>
  );
}

export default App;
