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
        const user = await res_user.json();
        const session: Session = await res_session.json();
        await store.dispatch("user/setUsername", user.name, { root: true });
        await store.dispatch(
          "session/setSession",
          { id: session_id, name: session.name, status: session.status },
          { root: true }
        );
        await store.dispatch("user/setUserID", String(user_id), { root: true });
        await store.dispatch("portfolio/setPowerPlants", {}, { root: true });
        await store.dispatch("session/updateBidAbility", user.can_bid, {
          root: true,
        });
        if (session.phase_infos !== null) {
          await store.commit("session/SET_PHASE_INFOS", session.phase_infos, {
            root: true,
          });
        }
      }
      next("/");
    },
  },
];

const router = new VueRouter({
  mode: "history",
  base: process.env.BASE_URL,
  routes,
});

export default router;
