import { useEffect } from "react";
import { toast } from "sonner";

let errorAlreadyFired = false;
let infoAlreadyFired = false;
export const useLoginError = () => {
  useEffect(() => {
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

      const info = JSON.parse(localStorage.getItem("info") || "{}");

      if (info.message && info.title) {
        setTimeout(() => {
          if (infoAlreadyFired) return;
          console.error(info);
          toast.info(info.title, {
            id: Math.random().toString(),
            description: info.message,
          });
          infoAlreadyFired = true;
          localStorage.removeItem("info");
        }, 50);
      }
    } catch (e) {
      console.error("Error not found while error=true set");
    }
  }, []);
};
