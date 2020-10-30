<template>
  <div class="number_input">
    <button
      class="btn btn_minus"
      v-if="show_btns"
      @click="emitUpdate(Number(value) - step)"
    >
      -
    </button>
    <input
      type="number"
      :value="value"
      inputmode="decimal"
      @change="e => emitUpdate(Number(e.target.valueAsNumber))"
    />
    <button
      class="btn btn_plus"
      v-if="show_btns"
      @click="emitUpdate(Number(value) + step)"
    >
      +
    </button>
  </div>
</template>

<script lang='ts'>
import { Component, Vue, Prop } from "vue-property-decorator";

@Component
export default class NumberInput extends Vue {
  @Prop() value!: number;
  @Prop({ default: 10 }) step!: number;
  @Prop({ default: true }) show_btns!: boolean;

  emitUpdate(value): void {
    console.log('value updated')
    this.$emit("input", Number(value));
  }
}
</script>

<style scoped>
.number_input {
  height: 2rem;
  display: flex;
  flex-direction: row;
  align-items: center;
}
.number_input > button {
  height: 2rem;
  width: 2rem;
  box-sizing: border-box;
  font-size: 1rem;
  margin: 0 0.5rem;
  border: 1px solid rgb(146, 146, 146);
  border-radius: 1rem;
}
.number_input > input[type="number"] {
  height: 2rem;
  box-sizing: border-box;
  border: 1px solid black;
  font-size: 1.7rem;
  text-align: center;
}

.number_input > input[type="number"] {
  -moz-appearance: textfield;
}

.number_input > input::-webkit-outer-spin-button,
.number_input > input::-webkit-inner-spin-button {
  -webkit-appearance: none;
}
</style>