import { createRouter, createWebHashHistory } from "vue-router";
import CommandsPage from "@/pages/CommandsPage.vue";
import SettingsPage from "@/pages/SettingsPage.vue";

export const routePaths = {
  home: "/",
  groups: "/groups",
  commands: "/commands",
  categories: "/categories",
  settings: "/settings",
};

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    { path: routePaths.home, name: "dashboard", component: CommandsPage },
    { path: routePaths.commands, name: "commands", component: CommandsPage },
    { path: routePaths.settings, name: "settings", component: SettingsPage },
  ],
});

export default router;
export type TRoutePaths = keyof typeof routePaths;
