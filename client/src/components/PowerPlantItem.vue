<template>
  <div class="pp__grid">
    <div class="pp__logo">{{ logo }}</div>
    <div :style="style_barre" ref="barre">
      <input
        type="range"
        class="pp__barre__slider slider__active"
        v-model="power_plant.planning_modif"
        :min="p_min"
        :max="power_plant.p_max_mw"
        step="10"
        :disabled="!editable || power_plant.type === 'ren'"
      />
      <div class="pp__barre__p_planning" :style="style_pp__barre__p_planning" />
      <div class="pp__barre__p_min" :style="style_pp__barre__p_min" />
      <div class="pp__barre__p_max" />
      <div
        class="pp__barre__current_p_max"
        :style="style_pp__barre__current_p_max"
      />
      <div
        class="pp__barre__current_p_min"
        :style="style_pp__barre__current_p_min"
      />
    </div>
    <div class="pp__barre__legend" :style="style_barre_pmax_width">
      <div class="pp__barre__legend__pmin" :style="style_legend_pmin">
        {{ power_plant.p_min_mw }} MW
      </div>
      <div class="pp__barre__legend__p" :style="style_legend_p">
        {{ power_plant.planning_modif }} MW
      </div>
      <div class="pp__barre__legend__pmax" :style="style_legend_pmax">
        {{ power_plant.p_max_mw }} MW
      </div>
    </div>
    <div class="pp__infos">
      <span>
        Coût :
        <strong>{{ power_plant.price_eur_per_mwh }}</strong> €/MWh,
      </span>
      <span>
        Stock :
        <strong
          >{{ stock }}
          {{ stock === "∞" ? "" : " / " + power_plant.stock_max_mwh }}</strong
        >
        {{ stock === "∞" ? "" : "MWh" }}
      </span>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue, Watch } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { PowerPlant } from "../store/portfolio";

@Component
export default class PowerPlantItem extends Vue {
  @Prop() power_plant!: PowerPlant;
  @Prop() power_max_mw!: number;
  @Prop() editable!: boolean;

  /**
   * Watcher to check user's power plant setpoint.
   */
  get current_p_max(): number {
    // Max power taking into account remaining stock if applicable
    const stock =
      this.power_plant.stock_mwh === -1
        ? Number.POSITIVE_INFINITY
        : this.power_plant.stock_mwh;
    return Math.max(0, Math.min(this.power_plant.p_max_mw, stock));
  }
  get current_stock_p_min(): number {
    // Min power taking into account free remaining stock if applicable
    if (this.power_plant.type === "storage") {
      const stock =
        this.power_plant.stock_mwh === -1
          ? Number.POSITIVE_INFINITY
          : this.power_plant.stock_mwh;
      return Math.max(
        -this.power_plant.p_max_mw,
        -(this.power_plant.stock_max_mwh - stock)
      );
    } else {
      return 0;
    }
  }
  @Watch("power_plant.planning_modif")
  onValueUpdate(new_val: number, old_val: number): void {
    if (Math.abs(new_val) < this.power_plant.p_min_mw) {
      this.power_plant.planning_modif = 0;
    }
    if (new_val > this.current_p_max) {
      this.power_plant.planning_modif = this.current_p_max;
    }
    if (new_val < this.current_stock_p_min) {
      this.power_plant.planning_modif = this.current_stock_p_min;
    }
  }

  get p_min(): number {
    return this.power_plant.type === "storage" ? -this.power_plant.p_max_mw : 0;
  }

  /**
   * Convert power values into width fractions.
   */
  get p_max_abs_ratio(): number {
    // Size of the current PP barre relative to the size of the PP with the highest PMax
    return (this.power_plant.p_max_mw / this.power_max_mw) * 100;
  }
  get p_max_ratio(): number {
    return (this.power_plant.p_max_mw / this.power_plant.p_max_mw) * 100;
  }
  get current_p_max_ratio(): number {
    return Math.min(
      100,
      Math.max(
        0,
        (1 -
          (this.current_p_max - this.p_min) /
            (this.power_plant.p_max_mw - this.p_min)) *
          100
      )
    );
  }
  get current_p_min_ratio(): number {
    return Math.min(
      100,
      Math.max(
        0,
        ((this.current_stock_p_min - this.p_min) /
          (this.power_plant.p_max_mw - this.p_min)) *
          100
      )
    );
  }
  get p_min_active_ratio(): number {
    return (
      (this.power_plant.p_min_mw / (this.power_plant.p_max_mw - this.p_min)) *
      100
    );
  }
  get p_planning_ratio(): number {
    return (
      ((this.power_plant.planning - this.p_min) /
        (this.power_plant.p_max_mw - this.p_min)) *
      100
    );
  }
  get p_value_ratio(): number {
    return (
      ((this.power_plant.planning_modif - this.p_min) /
        (this.power_plant.p_max_mw - this.p_min)) *
      100
    );
  }

  /**
   * Monitor component's width to update visibility ratio.
   */
  content_width = 0;
  public handleResize() {
    if (this.$refs.barre)
      this.content_width = (this.$refs.barre as HTMLDivElement).clientWidth;
  }
  get visibility_ratio(): number {
    return (85 / Number(this.content_width)) * 100;
  }
  public mounted() {
    window.addEventListener("resize", this.handleResize);
    if (this.$refs.barre)
      this.content_width = (this.$refs.barre as HTMLDivElement).clientWidth;
  }
  public beforeDestroyed() {
    window.removeEventListener("resize", this.handleResize);
  }

  /**
   * Logos and format stock string.
   */
  get stock(): string {
    return this.power_plant.stock_mwh === -1
      ? "∞"
      : String(this.power_plant.stock_mwh);
  }
  get logo(): string {
    let logo = "";
    if (this.power_plant.type === "nuc") logo = "☢️";
    if (this.power_plant.type === "therm") logo = "🔥";
    if (this.power_plant.type === "hydro") logo = "💧";
    if (this.power_plant.type === "ren") logo = "☀️";
    if (this.power_plant.type === "storage") logo = "🔋";
    return logo;
  }

  /**
   * Dynamic styles.
   */
  get style_barre(): string {
    return `
      position: relative;
      box-sizing: content-box;
      grid-area: barre;
      border: 2px solid rgb(0, 195, 255);
      border-radius: 2px;
      width: ${this.p_max_abs_ratio}%
    `;
  }
  get style_barre_pmax_width(): string {
    return `
      width: ${this.p_max_abs_ratio}%;
    `;
  }
  get style_pp__barre__current_p_max(): string {
    let style = `
      width: ${this.current_p_max_ratio}%;
      display: ${this.current_p_max_ratio === 0 ? "none" : "block"};
    `;
    if (this.current_p_max <= this.power_plant.p_min_mw) {
      style += "border-left: none;";
    } else {
      style += "border-left: 2px dashed black;";
    }
    return style;
  }
  get style_pp__barre__current_p_min(): string {
    let style = `
      width: ${this.current_p_min_ratio}%;
      display: ${this.current_p_min_ratio === 0 ? "none" : "block"};
    `;
    if (this.current_stock_p_min >= -this.power_plant.p_min_mw) {
      style += "border-right: none;";
    } else {
      style += "border-right: 2px dashed black;";
    }
    return style;
  }
  get style_pp__barre__p_min(): string {
    if (this.power_plant.type !== "storage") {
      return `
      width: ${this.p_min_active_ratio}%;
      display: ${this.p_min_active_ratio === 0 ? "none" : "block"};
      border-right: 2px dashed black;
      left: 0;
    `;
    } else {
      let style = `
        width: ${this.p_min_active_ratio * 2}%;
        display: ${this.p_min_active_ratio === 0 ? "none" : "block"};
        left: calc(${50 - this.p_min_active_ratio}% - 2px);
      `;
      if (this.current_stock_p_min >= -this.power_plant.p_min_mw) {
        style += "border-left: none;";
      } else {
        style += "border-left: 2px dashed black;";
      }
      if (this.current_p_max <= this.power_plant.p_min_mw) {
        style += "border-right: none;";
      } else {
        style += "border-right: 2px dashed black;";
      }
      return style;
    }
  }
  get style_pp__barre__p_planning(): string {
    return `
      width: ${this.p_planning_ratio}%;
      display: ${this.power_plant.type === "ren" ? "none" : "block"}
    `;
  }
  get style_legend_pmin(): string {
    if (this.power_plant.type !== "storage") {
      return `
      position: absolute;
      left: calc(${this.p_min_active_ratio}% - 75px);
      display: ${
        Math.abs(this.p_min_active_ratio - this.p_value_ratio) <
        this.visibility_ratio
          ? "none"
          : "block"
      }
    `;
    } else {
      return "display: none;";
    }
  }
  get style_legend_pmax(): string {
    return `
      position: absolute;
      left: calc(${this.p_max_ratio}% - ${
      this.power_plant.p_max_mw === this.power_max_mw ? 90 : 75
    }px);
      display: ${
        Math.abs(this.p_max_ratio - this.p_value_ratio) < this.visibility_ratio
          ? "none"
          : "block"
      }
    `;
  }
  get style_legend_p(): string {
    return `
      position: absolute;
      left: calc(${this.p_value_ratio}% - 75px);
    `;
  }
}
</script>

<style scoped>
@media screen and (max-width: 400px) {
  .pp__grid {
    display: grid;
    grid-template-areas:
      "logo infos"
      "legend legend"
      "barre barre";
    grid-template-columns: 50px 1fr;
    grid-template-rows: 45px 30px 60px;
  }

  .pp__logo {
    padding-bottom: 10px;
  }
  .pp__infos {
    display: flex;
    flex-direction: column;
  }
}
@media screen and (min-width: 400px) {
  .pp__grid {
    display: grid;
    grid-template-areas:
      "vide infos"
      "logo barre"
      "X legend";
    grid-template-columns: 50px 1fr;
    grid-template-rows: 20px 50px 30px;
  }
}

.pp__logo {
  grid-area: logo;
  align-self: center;
  font-size: 2rem;
}

.pp__barre__p_planning {
  border-right: 3px solid rgba(255, 0, 0, 0.568);
  height: calc(100% + 4px);
  position: absolute;
  top: -2px;
  left: 0px;
  z-index: 2;
  box-sizing: border-box;
}

.pp__barre__p_max {
  position: absolute;
  top: 0;
  right: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
}
.pp__barre__p_min,
.pp__barre__current_p_max,
.pp__barre__current_p_min {
  background: repeating-linear-gradient(
    -45deg,
    #c8cad4a9,
    #c8cad4a9 5px,
    #a2a4aaa9 5px,
    #a2a4aaa9 10px
  );
}
.pp__barre__p_min {
  position: absolute;
  height: 100%;
  text-align: end;
}
.pp__barre__current_p_max {
  position: absolute;
  height: 100%;
  right: 0;
  bottom: 0;
}
.pp__barre__current_p_min {
  position: absolute;
  height: 100%;
  left: 0;
  bottom: 0;
}

.pp__barre__legend {
  position: relative;
  grid-area: legend;
  font-weight: bold;
  padding-top: 3px;
}
.pp__barre__legend > * {
  width: 150px;
}

.pp__infos {
  grid-area: infos;
  align-self: center;
  margin-bottom: 5px;
  text-align: start;
}

/*-------------------- Input range default styling ----------------------*/

@media screen and (max-width: 400px) {
  /* Cannot merge selectors */
  input[type="range"] {
    height: 60px;
  }
  input[type="range"]::-webkit-slider-thumb {
    height: 60px;
  }
  input[type="range"]::-moz-range-thumb {
    height: 60px;
  }
  input[type="range"]::-ms-thumb {
    height: 60px;
  }
}
@media screen and (min-width: 400px) {
  /* Cannot merge selectors */
  input[type="range"] {
    height: 50px;
  }
  input[type="range"]::-webkit-slider-thumb {
    height: 50px;
  }
  input[type="range"]::-moz-range-thumb {
    height: 50px;
  }
  input[type="range"]::-ms-thumb {
    height: 50px;
  }
}

.pp__barre__slider {
  position: absolute;
  left: 0;
  bottom: -1px;
  z-index: 3;
  width: 100%;
  margin: 0;
}
/* Taken from https://css-tricks.com/styling-cross-browser-compatible-range-inputs-css/ */
input[type="range"] {
  -webkit-appearance: none; /* Hides the slider so that custom slider can be made */
  width: 100%; /* Specific width is required for Firefox. */
  background: transparent; /* Otherwise white in Chrome */
}

input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
}

input[type="range"]:focus {
  outline: none; /* Removes the blue border. You should probably do some kind of focus styling for accessibility reasons though. */
}

input[type="range"]::-ms-track {
  width: 100%;
  cursor: pointer;

  /* Hides the slider so custom styles can be added */
  background: transparent;
  border-color: transparent;
  color: transparent;
}

/*-------------------- Default range thumb styling ----------------------*/
/* Special styling for WebKit/Blink */
input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 4px;
  box-sizing: content-box;
  border-radius: 3px;
  cursor: pointer;
  margin-top: 0px; /* You need to specify a margin in Chrome, but in Firefox and IE it is automatic */
}

/* All the same stuff for Firefox */
input[type="range"]::-moz-range-thumb {
  width: 4px;
  border-radius: 3px;
  box-sizing: content-box;
  cursor: pointer;
}

/* All the same stuff for IE */
input[type="range"]::-ms-thumb {
  width: 4px;
  border-radius: 3px;
  box-sizing: content-box;
  cursor: pointer;
}

/*-------------------- Active range thumb styling ----------------------*/

input[type="range"].slider__active::-webkit-slider-thumb {
  box-shadow: 1px 1px 1px #000000, 0px 0px 1px #0d0d0d;
  border: 1px solid #000000;
  background: #ffffff;
}

/* All the same stuff for Firefox */
input[type="range"].slider__active::-moz-range-thumb {
  box-shadow: 1px 1px 1px #000000, 0px 0px 1px #0d0d0d;
  border: 1px solid #000000;
  background: #ffffff;
}

/* All the same stuff for IE */
input[type="range"].slider__active::-ms-thumb {
  box-shadow: 1px 1px 1px #000000, 0px 0px 1px #0d0d0d;
  border: 1px solid #000000;
  background: #ffffff;
}
</style>
