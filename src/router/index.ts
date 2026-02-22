import { createRouter, createWebHashHistory } from "vue-router";
import App from "../App.vue";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      component: App,
    },
    {
      path: "/mini",
      component: () => import("../views/MiniWindow.vue"),
    },
  ],
});

export default router;
