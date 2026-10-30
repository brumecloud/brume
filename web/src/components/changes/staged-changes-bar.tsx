import { useLiveSuspenseQuery } from "@tanstack/react-db";
import { useMutation } from "@tanstack/react-query";
import { useParams } from "@tanstack/react-router";
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

// Reference: the "Apply 2 changes" bar of Railway. Floats at the top of the canvas while changes
// wait, centered on what the service panel leaves free.
export function StagedChangesBar({ environmentId }: { readonly environmentId: EnvironmentId }) {
  const { data: changes } = useLiveSuspenseQuery({ query: environmentChanges(environmentId) });
  const { data: services } = useLiveSuspenseQuery({ query: environmentServices(environmentId) });
  const deployment = useMutation({ mutationFn: () => deployChanges(environmentId) });
  const discard = useMutation({ mutationFn: () => discardChanges(environmentId) });
  const reduceMotion = useReducedMotion();
  const { serviceId } = useParams({ strict: false });
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

  // Too narrow for both, the bar starts on the left and runs over the panel.
  return (
    <div
      data-panel={serviceId ? "open" : "closed"}
      className="pointer-events-none absolute inset-x-0 top-3 z-20 flex justify-center-safe px-4 md:data-[panel=open]:right-panel"
    >
      <AnimatePresence>
        {pending && (
          <motion.div
            role="region"
            aria-label="Changements en attente"
            initial={{ opacity: 0, transform: reduceMotion ? "none" : "translateY(-8px)" }}
            animate={{ opacity: 1, transform: "translateY(0px)" }}
            exit={{ opacity: 0, transform: reduceMotion ? "none" : "translateY(-8px)" }}
            transition={PANEL_TRANSITION}
            className="pointer-events-auto flex shrink-0 items-center gap-2 rounded-xl border border-info/40 bg-card py-1.5 ps-4 pe-1.5 shadow-2xl"
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
    </div>
  );
}
