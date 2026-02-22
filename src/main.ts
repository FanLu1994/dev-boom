import { createApp } from "vue";
import RootApp from "./RootApp.vue";
import router from "./router";
import "./styles/app.css";

createApp(RootApp).use(router).mount("#app");
