import Vue from "vue";
import VueRouter, { RouteConfig } from "vue-router";
import Home from "../views/Home.vue";
import store from "../store/";

Vue.use(VueRouter);

const routes: Array<RouteConfig> = [
  {
    path: "/",
    name: "Home",
    component: Home,
  },
  {
    path: "/auction/:auction_id/user/:user_id",
    beforeEnter: async (to, from, next) => {
      const auction_id: string = to.params.auction_id;
      const user_id: string = to.params.user_id;

      const res_user = await fetch(
        `${process.env.VUE_APP_API_URL}/auction/${auction_id}/user/${user_id}`,
        {
          method: "GET",
        }
      );
      const res_auction = await fetch(
        `${process.env.VUE_APP_API_URL}/auction/${auction_id}`,
        {
          method: "GET",
        }
      );
      if (res_user.status === 200 && res_auction.status === 200) {
        const user = await res_user.json();
        const auction = await res_auction.json();
        await store.dispatch("user/setUsername", user.name, { root: true });
        await store.dispatch(
          "auction/setAuction",
          { id: auction_id, name: auction.name, status: auction.status },
          { root: true }
        );
        await store.dispatch("user/setUserID", String(user_id), { root: true });
        await store.dispatch("auction/updateBidAbility", user.can_bid, {
          root: true,
        });
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
