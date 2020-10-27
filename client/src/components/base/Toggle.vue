<template>
  <div class="toggle__container">
    <button
      class="toggle toggle__left"
      :class="{
        toggle__active: value === left_value,
        toggle__inactive: value !== left_value
      }"
      @click="updateCallback(left_value)"
    >
      {{ left_label }}
    </button>
    <button
      class="toggle toggle__right"
      :class="{
        toggle__active: value === right_value,
        toggle__inactive: value !== right_value
      }"
      @click="updateCallback(right_value)"
    >
      {{ right_label }}
    </button>
  </div>
</template>

<script lang='ts'>
import { Component, Vue, Prop } from "vue-property-decorator";

@Component
export default class ToggleSwitch extends Vue {
  @Prop({ default: false }) value!: boolean;
  @Prop({ default: "left" }) left_label!: string;
  @Prop({ default: "left" }) left_value!: any;
  @Prop({ default: "right" }) right_label!: string;
  @Prop({ default: "left" }) right_value!: any;

  updateCallback(state: any): void {
    this.$emit("input", state);
  }
}
</script>

<style scoped>
.toggle__container {
  display: grid;
  grid-template-columns: 1fr 1fr;
}

button.toggle {
  border: none;
  font-size: 0.9rem;
}

button.toggle__left {
  border-bottom-left-radius: 8px 8px;
  border-top-left-radius: 8px 8px;
  padding: 5px;
}
button.toggle__right {
  border-bottom-right-radius: 8px 8px;
  border-top-right-radius: 8px 8px;
  padding: 5px;
}

button.toggle__active {
  border: 1px solid rgb(92, 92, 92);
  background-color: rgb(243, 243, 243);
  font-weight: bold;
  color: black;
  transition: 0.3s;
}
button.toggle__inactive {
  border: 1px solid rgb(177, 177, 177);
  background-color: white;
  font-weight: normal;
  color: rgb(88, 88, 88);
  transition: 0.3s;
}
</style>