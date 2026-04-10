import { useActiveElement, useMagicKeys, whenever } from "@vueuse/core";
import { computed, type Ref, ref } from "vue";
import { useRoute } from "vue-router";
import { routePaths } from "@/router";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import { ICommand } from "@/lib/api/api.types.ts";
import { commandsApi } from "@/lib/api/api.tauri.ts";
import { toast } from "vue-sonner";

export function createCommandHotKeys(dialogOpen: Ref<boolean>) {
  const route = useRoute();
  const keys = useMagicKeys();
  const activeElement = useActiveElement();

  const command = ref<ICommand | undefined>();

  const isTyping = computed(() =>
    ["INPUT", "TEXTAREA", "CONTENTEDITABLE"].includes(
      activeElement.value?.tagName || ""
    )
  );

  const isCorrectPath = computed(
    () =>
      !route.params.id &&
      [routePaths.home, routePaths.groups, routePaths.commands].includes(
        route.path
      )
  );

  const noDialogOpen = computed(() => {
    return (
      document.querySelectorAll(
        '[role="dialog"][data-state="open"], [role="alertdialog"][data-state="open"]'
      ).length === 0
    );
  });

  const shouldOpen = computed(
    () =>
      keys.ctrl_v.value &&
      isCorrectPath.value &&
      !isTyping.value &&
      noDialogOpen.value
  );

  whenever(shouldOpen, async () => {
    const text = await readText();
    if (!text) return;
    toast.success("Copied from clipboard");

    try {
      const result = await commandsApi.explain(text);
      command.value = {
        position: 0,
        id: 0,
        name: result.summary,
        command: text,
      };
    } catch (error) {
      toast.error("Something went wrong");
      console.error("Failed to fetch command explanation:", error);
    }

    dialogOpen.value = true;
  });

  return { command };
}
