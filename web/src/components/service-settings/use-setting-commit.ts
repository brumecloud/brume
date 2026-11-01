import { useMutation } from "@tanstack/react-query";

import { stageChange, unstageChange } from "~/api/changes";
import type {
  ServiceId,
  ServiceSettings,
  SettingField,
  SettingValue,
  StagedChange,
} from "~/api/types";
import { settingChange, settingValue } from "~/helpers/changes";

// Editing a field stages a draft, or drops the draft when the value comes back to the current one.
export function useSettingCommit(
  serviceId: ServiceId,
  settings: ServiceSettings,
  changes: readonly StagedChange[],
) {
  const mutation = useMutation({ mutationFn: stageChange });

  const commit = (field: SettingField, value: SettingValue) => {
    const decision = settingChange(field, value, settingValue(settings, field), changes);
    if (decision.action === "stage") {
      mutation.mutate({ serviceId, type: "setting", field, value });
    } else if (decision.action === "unstage") {
      unstageChange(decision.changeId);
    }
  };

  // The field whose change was refused, so that the form can show the refusal next to it.
  const failed = mutation.error ? mutation.variables : undefined;
  const failedField = failed?.type === "setting" ? failed.field : undefined;
  return { commit, error: mutation.error, failedField };
}
