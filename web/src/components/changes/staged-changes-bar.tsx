import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useMutation } from "@tanstack/react-query";
import { EllipsisVerticalIcon } from "lucide-react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { useEffect } from "react";

import { deployChanges, discardChanges, environmentChanges } from "~/api/changes";
import { environmentServices } from "~/api/services";
import type { EnvironmentId } from "~/api/types";
import { Button } from "~/components/ui/button";
import { Kbd, KbdGroup } from "~/components/ui/kbd";
import { Menu, MenuItem, MenuPopup, MenuTrigger } from "~/components/ui/menu";
import { changeCountLabel } from "~/helpers/changes";
import { PANEL_TRANSITION } from "~/lib/motion";

import { ChangesDialog } from "./changes-dialog";

// Reference: the "Apply 2 changes" bar of Railway. Sits in the header while changes wait, so it
// never covers the panel.
export function StagedChangesBar({ environmentId }: { readonly environmentId: EnvironmentId }) {
  const { data: changes } = useLiveSuspenseQuery({ query: environmentChanges(environmentId) });
  const { data: services } = useLiveSuspenseQuery({ query: environmentServices(environmentId) });
  const deployment = useMutation({ mutationFn: () => deployChanges(environmentId) });
  const discard = useMutation({ mutationFn: () => discardChanges(environmentId) });
  const reduceMotion = useReducedMotion();
  const { mutate: deployAll } = deployment;
  const pending = changes.length > 0;

  // oxlint-disable-next-line brume/no-unreasoned-effect -- listens to Shift+Enter on the whole page while changes wait
  useEffect(() => {
    if (!pending) {
      return undefined;
    }
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.shiftKey && event.key === "Enter") {
        event.preventDefault();
        deployAll();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [pending, deployAll]);

  return (
    <AnimatePresence>
      {pending && (
        <motion.div
          role="region"
          aria-label="Changements en attente"
          initial={{ opacity: 0, transform: reduceMotion ? "none" : "translateY(-6px)" }}
          animate={{ opacity: 1, transform: "translateY(0px)" }}
          exit={{ opacity: 0, transform: reduceMotion ? "none" : "translateY(-6px)" }}
          transition={PANEL_TRANSITION}
          className="flex items-center gap-2 rounded-xl border border-info/40 bg-card py-1 ps-3 pe-1"
        >
          <span className="me-3 font-medium text-info-foreground text-sm">
            {changeCountLabel(changes.length)}
          </span>
          {deployment.error && (
            <span role="alert" className="text-destructive-foreground text-sm">
              {deployment.error.message}
            </span>
          )}
          <ChangesDialog changes={changes} services={services} />
          <Button size="sm" loading={deployment.isPending} onClick={() => deployAll()}>
            Déployer
            <KbdGroup>
              <Kbd>⇧</Kbd>
              <Kbd>Entrée</Kbd>
            </KbdGroup>
          </Button>
          <Menu>
            <MenuTrigger
              render={<Button variant="ghost" size="icon-sm" aria-label="Autres actions" />}
            >
              <EllipsisVerticalIcon />
            </MenuTrigger>
            <MenuPopup align="end">
              <MenuItem variant="destructive" onClick={() => discard.mutate()}>
                Annuler tous les changements
              </MenuItem>
            </MenuPopup>
          </Menu>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
