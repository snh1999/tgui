import { defineStore } from "pinia";
import { ref } from "vue";

export type TViewMode = "list" | "grid" | "table";

export const useCommandsStore = defineStore("commandsStore", () => {
  const search = ref("");
  const filterCategory = ref("all");
  const showFavoritesOnly = ref(false);
  const showRunningOnly = ref(false);
  const view = ref<TViewMode>("list");

  return { search, filterCategory, showFavoritesOnly, showRunningOnly, view };
});
