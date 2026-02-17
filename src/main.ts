import { createApp } from "vue";
import App from "./App.vue";
import "./style.css";
import "vue-sonner/style.css";
import "./themes/catppuccin.css";

import { VueQueryPlugin } from "@tanstack/vue-query";
import { apiClient } from "@/lib/api/api.client.ts";
import router from "@/router";

const app = createApp(App);
app.use(router);
app.use(VueQueryPlugin, {
  queryClient: apiClient,
});
app.mount("#app");
