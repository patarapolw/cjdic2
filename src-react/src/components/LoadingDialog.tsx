import { useEffect, useState } from "react";

import { Dialog, Portal } from "@chakra-ui/react";
import { invoke } from "@tauri-apps/api/core";

function LoadingDialog() {
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    invoke<boolean>("is_yomitan_setup_yet").then((is_setup) =>
      setIsLoading(!is_setup),
    );
  }, []);

  return (
    <Dialog.Root open={isLoading}>
      <Dialog.Trigger />
      <Portal>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content>
            <Dialog.CloseTrigger />
            <Dialog.Header>
              <Dialog.Title />
            </Dialog.Header>
            <Dialog.Body>Example text</Dialog.Body>
            <Dialog.Footer />
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  );
}

export default LoadingDialog;
