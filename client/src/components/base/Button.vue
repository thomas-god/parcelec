<template>
  <button @click="emitClilck" :style="style">
    <slot>OK</slot>
  </button>
</template>

<script lang='ts'>
import { Component, Prop, Vue, Watch } from "vue-property-decorator";

@Component
export default class CustomButton extends Vue {
  @Prop({ default: "rgb(0, 132, 255)" }) background_color!: string;
  @Prop({ default: "white" }) text_color!: string;
  @Prop({ default: "1rem" }) font_size!: string;
  @Prop({ default: false }) disabled!: boolean;

  get style(): string {
    let style = "";
    style += `background-color: ${this.background_color};`;
    style += `color: ${this.text_color};`;
    style += `opacity: ${this.disabled ? 0.5 : 1};`;
    style += `font-size: ${this.font_size};`
    return style;
  }

  emitClilck(): void {
    if (!this.disabled) this.$emit("click");
  }
}
</script>

<style scoped>
button {
  box-sizing: border-box;
  border: none;
  border-radius: 1rem;
  padding: 5px 1rem;
  font-weight: normal;
}
button:active {
  opacity: 0.5 !important;
}
</style>