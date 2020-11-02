import Vue from "vue";
import App from "./App.vue";
import router from "./router";
import store from "./store";

import { Chart } from "chart.js";

Chart.defaults.global.defaultFontFamily = '"Montserrat"';
Chart.defaults.global.defaultFontColor = "black";
Chart.defaults.global.defaultFontSize = 15;
Chart.defaults.global.defaultFontStyle = "normal";

Vue.config.productionTip = false;

new Vue({
  router,
  store,
  render: h => h(App)
}).$mount("#app");
