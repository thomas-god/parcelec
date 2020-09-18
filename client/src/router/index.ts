import Vue from "vue";
import VueRouter, { RouteConfig } from "vue-router";
import Home from "../views/Home.vue";
import store from "../store/";
import { Session } from "@/store/session";

Vue.use(VueRouter);

const routes: Array<RouteConfig> = [
  {
    path: "/",
    name: "Home",
    component: Home,
  },
  {
    path: "/session/:session_id/user/:user_id",
    component: Home,
    beforeEnter: async (to, from, next) => {
      const session_id: string = to.params.session_id;
      const user_id: string = to.params.user_id;

      const res_user = await fetch(
        `${process.env.VUE_APP_API_URL}/session/${session_id}/user/${user_id}`,
        {
          method: "GET",
        }
      );
      const res_session = await fetch(
        `${process.env.VUE_APP_API_URL}/session/${session_id}`,
        {
          method: "GET",
        }
      );
      if (res_user.status === 200 && res_session.status === 200) {
        await store.dispatch("session/setSessionID", session_id, {
          root: true,
        });
        await store.dispatch("user/setUserID", user_id, { root: true });
        store.dispatch("session/loadGameContent", {}, { root: true });
      }
      next();
    },
  },
];

const router = new VueRouter({
  mode: "history",
  base: process.env.BASE_URL,
  routes,
});

export default router;
