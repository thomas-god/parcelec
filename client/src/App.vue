<template>
  <div id="app">
    <navbar />
    <router-view />
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import Navbar from "./components/Navbar.vue";

@Component({
  components: {
    Navbar
  }
})
export default class App extends Vue {}
</script>

<style>
html {
  height: 100%;
}
body {
  margin: 0;
  height: 100%;
}
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  height: 100%;
  overflow: auto;
}
</style>
