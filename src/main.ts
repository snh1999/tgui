import {createApp} from "vue";
import App from "./App.vue";
import "./style.css";
import "./themes/catppuccin.css"
import router from "@/router";

const app = createApp(App);
app.use(router);
app.mount("#app");
