import "./App.css";

import LoadingDialog from "./components/LoadingDialog";
import { Provider } from "./components/ui/provider";
import SearchPage from "./pages/SearchPage";

function App() {
  return (
    <Provider>
      <SearchPage />
      <LoadingDialog />
    </Provider>
  );
}

export default App;
