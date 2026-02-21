import "./App.css";

import { useEffect, useState } from "react";

import { Button, Field, Input, Stack } from "@chakra-ui/react";
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
    invoke("add_entry", { name });
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  }

  return (
    <Provider>
      <Stack maxW={"1000px"} margin={"0.5em auto"}>
        <Field.Root
          orientation={"horizontal"}
          onSubmit={(e) => {
            e.preventDefault();
            greet();
          }}
        >
          <Input
            id="greet-input"
            onChange={(e) => setName(e.currentTarget.value)}
            autoComplete="off"
            placeholder="Enter a name..."
          />
          <Button>Search</Button>
        </Field.Root>
        <p>{greetMsg}</p>
      </Stack>
    </Provider>
  );
}

export default App;
