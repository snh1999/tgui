import { defineStore } from "pinia";
import { ref } from "vue";

export const useCommandsViewStore = defineStore("commandsView", () => {
  const search = ref("");
  const filterCategory = ref("all");
  const showFavoritesOnly = ref(false);
  const showRunningOnly = ref(false);
  const view = ref<"list" | "grid" | "table">("list");

  return { search, filterCategory, showFavoritesOnly, showRunningOnly, view };
});
