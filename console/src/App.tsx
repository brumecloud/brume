import {
  RouterProvider,
  createBrowserRouter,
} from "react-router-dom";
import { Toaster } from "sonner";

import { Router } from "./router/router";

function App() {
  return (
    <>
      <RouterProvider router={createBrowserRouter(Router)} />
      <Toaster
        richColors
        toastOptions={{
          duration: 4000,
        }}
      />
    </>
  );
}

export default App;
