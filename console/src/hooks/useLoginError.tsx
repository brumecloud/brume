import { useEffect } from "react";
import { toast } from "sonner";

let errorAlreadyFired = false;
export const useLoginError = () => {
  useEffect(() => {
    if (!window.location.search.includes("error=true")) {
      return;
    }

    try {
      const error = JSON.parse(localStorage.getItem("error") || "{}");

      if (error.message && error.title) {
        setTimeout(() => {
          if (errorAlreadyFired) return;
          console.error(error);
          toast.error(error.title, {
            id: Math.random().toString(),
            description: error.message,
          });
          errorAlreadyFired = true;
          localStorage.removeItem("error");
        }, 50);
      }
    } catch (e) {
      console.error("Error not found while error=true set");
    }
  }, []);
};
