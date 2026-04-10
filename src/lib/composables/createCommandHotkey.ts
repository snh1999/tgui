import { readText } from "@tauri-apps/plugin-clipboard-manager";
import { useActiveElement, useMagicKeys, whenever } from "@vueuse/core";
import { computed, type Ref, ref, watch } from "vue";
import { useRoute } from "vue-router";
import { toast } from "vue-sonner";
import { commandsApi } from "@/lib/api/api.tauri.ts";
import type { ICommand } from "@/lib/api/api.types.ts";
import { routePaths } from "@/router";

export function createCommandHotKeys(dialogOpen: Ref<boolean>) {
  const route = useRoute();
  const keys = useMagicKeys();
  const activeElement = useActiveElement();

  const command = ref<ICommand | undefined>();

  const isTyping = computed(() => {
    return ["INPUT", "TEXTAREA"].includes(activeElement.value?.tagName || "");
  });

  const isCorrectPath = computed(
    () =>
      !route.params.id &&
      [routePaths.home, routePaths.groups, routePaths.commands].includes(
        route.path
      )
  );

  const hasOpenDialog = () => {
    return (
      document.querySelectorAll(
        '[role="dialog"][data-state="open"], [role="alertdialog"][data-state="open"]'
      ).length > 0
    );
  };

  const shouldOpen = computed(
    () =>
      keys.ctrl_v.value &&
      isCorrectPath.value &&
      !isTyping.value &&
      !hasOpenDialog()
  );

  watch(dialogOpen, () => {
    if (!dialogOpen.value) {
      command.value = undefined;
    }
  });

  whenever(shouldOpen, async () => {
    try {
      const text = await readText();
      if (!text?.trim()) {
        return;
      }
      const result = await commandsApi.explain(text);
      command.value = {
        position: 0,
        id: 0,
        name: result.summary,
        command: text,
      };
      toast.success("Copied from clipboard");
    } catch (error) {
      toast.error("Something went wrong");
      console.error("Failed to fetch command explanation:", error);
      return;
    }

    dialogOpen.value = true;
  });

  return { command };
}
