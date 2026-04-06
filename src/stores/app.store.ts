import { defineStore } from "pinia";
import { ref } from "vue";

export type TLayoutState = "horizontal" | "vertical" | "full screen";
export const useAppStore = defineStore("appState", () => {
  const layoutState = ref<TLayoutState>("vertical");

  function toggleLayoutState() {
    if (layoutState.value === "horizontal") {
      layoutState.value = "vertical";
    } else if (layoutState.value === "vertical") {
      layoutState.value = "full screen";
    } else {
      layoutState.value = "horizontal";
    }
  }
  return { layoutState, toggleLayoutState };
});
