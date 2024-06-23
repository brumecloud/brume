import { BrowserRouter } from "react-router-dom";
import { Toaster } from "sonner";

import { Router } from "./router/router";

function App() {
  return (
    <BrowserRouter>
      <Router />
      <Toaster
        richColors
        toastOptions={{
          duration: 4000,
        }}
      />
    </BrowserRouter>
  );
}

export default App;
