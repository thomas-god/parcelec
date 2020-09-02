<template>
  <div class="home">
    <img alt="Vue logo" src="../assets/logo.png" />
    <MyComponent msg="Welcome to Your Vue.js App" />
  </div>
</template>

<script>
// @ is an alias to /src
import MyComponent from "@/components/MyComponent.vue";

export default {
  name: "Home",
  components: {
    MyComponent
  }
};
</script>
