import { createApp } from "vue";
import App from "./App.vue";
import "vue-sonner/style.css";
import "./themes/catppuccin.css";

import { VueQueryPlugin } from "@tanstack/vue-query";
import { createPinia } from "pinia";
import { apiClient } from "@/lib/api/api.client.ts";
import { initExecutionEvents } from "@/lib/api/api.events.ts";
import router from "@/router";

const app = createApp(App);
app.use(router);
app.use(VueQueryPlugin, {
  queryClient: apiClient,
});
app.use(createPinia());

// REASONS TO USE IT HERE
// 1. App.vue mount-remount can cause log events to appear twice. it runs exactly once now
// 2. the backend data hydrates before app mounts, so no risk of getting stale data
initExecutionEvents().catch((err) => {
  console.error("Failed to initialize execution events:", err);
});
app.mount("#app");
