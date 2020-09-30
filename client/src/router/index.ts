import Vue from "vue";
import VueRouter, { NavigationGuardNext, Route, RouteConfig } from "vue-router";
import Home from "../views/Home.vue";
import Main from "../components/Main.vue";
import SessionSelectMulti from "../components/SessionSelectMulti.vue";
import SessionSelectScenario from "../components/SessionSelectScenario.vue";
import UsernameSelect from "../components/UsernameSelect.vue";
import BilansEndGame from "../components/BilansEndGame.vue";
import store from "../store/";

Vue.use(VueRouter);

const uuid_regex =
  "[A-F0-9]{8}-[A-F0-9]{4}-4[A-F0-9]{3}-[89AB][A-F0-9]{3}-[A-F0-9]{12}";

const routes: Array<RouteConfig> = [
  {
    path: "/",
    name: "Home",
    component: Home,
  },
  {
    path: "/join",
    name: "Join",
    component: SessionSelectMulti,
  },
  {
    path: "/create",
    name: "Create",
    component: SessionSelectScenario,
  },
  {
    path: `/session/:session_id(${uuid_regex})`,
    component: UsernameSelect,
  },
  {
    path: `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})`,

    component: Main,
    beforeEnter: loadSession,
  },
  {
    path: `/session/:session_id(${uuid_regex})/user/:user_id(${uuid_regex})/results`,
    component: BilansEndGame,
    beforeEnter: loadSession,
  },
  { path: "*", redirect: "/" },
];

/**
 * Check if session and user exist and load the appropriate data into the store.
 * @param to
 * @param from
 * @param next
 */
async function loadSession(
  to: Route,
  from: Route,
  next: NavigationGuardNext<Vue>
) {
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
  } else {
    next("/");
  }
  next();
}

const router = new VueRouter({
  mode: "history",
  base: process.env.BASE_URL,
  routes,
});

export default router;
