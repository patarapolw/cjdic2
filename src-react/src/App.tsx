import "./App.css";

import { createBrowserRouter } from "react-router";
import { RouterProvider } from "react-router/dom";

import LoadingDialog from "./components/LoadingDialog";
import { Provider } from "./components/ui/provider";

const router = createBrowserRouter([
  {
    path: "/",
    lazy: async () => {
      return {
        Component: (await import("./pages/SearchPage")).default,
      };
    },
  },
]);

function App() {
  return (
    <Provider>
      <>
        <RouterProvider router={router} />
        <LoadingDialog />
      </>
    </Provider>
  );
}

export default App;
