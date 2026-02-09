<script lang="ts">
  import * as d3 from "d3";

  import { type Forecasts } from "../organisms/Forecasts.svelte";

  let {
    height,
    width,
    consumers_forecasts,
    consumers_history,
    renewables_forecasts,
    renewables_history,
  }: {
    width: number;
    height: number;
    consumers_forecasts: Forecasts;
    consumers_history: number[];
    renewables_forecasts: Forecasts;
    renewables_history: number[];
  } = $props();

  let marginTop = 20;
  let marginRight = 20;
  let marginBottom = 20;
  let marginLeft = 55;

  type PointType = "realized" | "active" | "forecast";
  type Source = "consumers" | "renewables";

  let gx: SVGGElement;
  let gy: SVGGElement;
  let gyGrid: SVGGElement;
  let gLowValues: SVGGElement;
  let gHighValues: SVGGElement;
  let gCurrentPeriod: SVGGElement;
  let svgElement: SVGElement;

  type Data = {
    source: Source;
    type: PointType;
    period: number;
    value: number;
    deviation: number;
  };

  let data = $derived.by(() => {
    const points: Data[] = [];

    // Add histories
    for (const [idx, value] of consumers_history.entries()) {
      points.push({
        source: "consumers",
        type: idx + 1 < consumers_history.length ? "realized" : "active",
        period: idx + 1,
        value: Math.abs(value),
        deviation: 0,
      });
    }

    for (const [idx, value] of renewables_history.entries()) {
      points.push({
        source: "renewables",
        type: idx + 1 < renewables_history.length ? "realized" : "active",
        period: idx + 1,
        value: Math.abs(value),
        deviation: 0,
      });
    }

    // Add forecasts
    for (const [key, value] of consumers_forecasts.entries()) {
      points.push({
        source: "consumers",
        type: "forecast",
        period: key,
        value: Math.abs(value.value),
        deviation: value.deviation,
      });
    }
    for (const [key, value] of renewables_forecasts.entries()) {
      points.push({
        source: "renewables",
        type: "forecast",
        period: key,
        value: Math.abs(value.value),
        deviation: value.deviation,
      });
    }
    return points;
  });

  let fx = $derived(
    d3
      .scaleBand()
      .domain(new Set(data.map((elem) => elem.period.toString())))
      .rangeRound([marginLeft, width - marginRight])
      .paddingInner(0.1)
      .paddingOuter(0.1),
  );
  let x = $derived(
    d3
      .scaleBand()
      .domain(new Set(data.map((elem) => elem.source)))
      .rangeRound([0, fx.bandwidth()])
      .paddingInner(0.05),
  );
  let y = $derived(
    d3
      .scaleLinear()
      .domain([
        0,
        d3.max(data, (elem) => elem.value + elem.deviation) as number,
      ])
      .nice()
      .rangeRound([height - marginBottom, marginTop]),
  );

  $effect(() => {
    // Draw x axis
    d3.select(gx).call((sel) =>
      sel
        .call(d3.axisBottom(fx))
        .attr("transform", `translate(0,${height - marginBottom})`),
    );

    // Draw y axis
    d3.select(gy).call((sel) =>
      sel
        .call(d3.axisLeft(y).tickFormat((d) => `${d} MW`))
        .attr("transform", `translate(${marginLeft},0)`),
    );

    // Draw y axis lines
    const yValues = y.ticks();
    d3.select(gyGrid).call((sel) =>
      sel
        .selectAll("line")
        .data(yValues)
        .join("line")
        .attr("x1", 0)
        .attr("x2", width - marginRight - marginLeft)
        .attr("y1", (tickValue) => y(tickValue))
        .attr("y2", (tickValue) => y(tickValue)),
    );

    // Draw faded rectangle (high-value forecasts)
    d3.select(gHighValues).call((sel) =>
      sel
        .selectAll("g")
        .data(d3.group(data, (elem) => elem.period))
        .join("g")
        .attr(
          "transform",
          ([period]) => `translate(${fx(period.toString())},0)`,
        )
        .selectAll("rect")
        .data(([_period, points]) => points)
        .join("rect")
        .attr("x", (point) => x(point.source) as number)
        .attr("y", (point) => y(point.value + point.deviation))
        .attr("width", x.bandwidth())
        .attr("height", (point) => y(0) - y(point.value + point.deviation))
        .attr("class", (point) => `${point.source}-forecast`)
        .attr("opacity", 1),
    );

    // Draw plain rectangles (low-value forecasts)
    d3.select(gLowValues).call((sel) =>
      sel
        .selectAll("g")
        .data(d3.group(data, (elem) => elem.period))
        .join("g")
        .attr(
          "transform",
          ([period]) => `translate(${fx(period.toString())},0)`,
        )
        .selectAll("rect")
        .data(([_period, points]) => points)
        .join("rect")
        .attr("x", (point) => x(point.source) as number)
        .attr("y", (point) => y(point.value - point.deviation))
        .attr("width", x.bandwidth())
        .attr("height", (point) => y(0) - y(point.value - point.deviation))
        .attr("opacity", (point) => (point.type === "realized" ? 0.1 : 1))
        .attr("class", (point) => point.source),
    );

    // Draw an outline for the current period
    d3.select(gCurrentPeriod).call((sel) =>
      sel
        .selectAll("g")
        .data(d3.group(data, (elem) => elem.period))
        .join("g")
        .attr(
          "transform",
          ([period]) => `translate(${fx(period.toString())},0)`,
        )
        .selectAll("rect")
        .data(([_period, points]) =>
          points.filter((elem) => elem.type === "active"),
        )
        .join("rect")
        .attr("x", (point) => x(point.source) as number)
        .attr("y", (point) => y(point.value - point.deviation))
        .attr("width", x.bandwidth())
        .attr("height", (point) => y(0) - y(point.value - point.deviation))
        .attr("stroke", "black")
        .attr("stroke-width", 1)
        .attr("fill", "none"),
    );
  });
</script>

<div class="flex flex-col gap-2">
  <svg
    {width}
    {height}
    viewBox={`0 0 ${width} ${height}`}
    role="img"
    class="h-full w-full select-none"
    bind:this={svgElement}
  >
    <g
      bind:this={gyGrid}
      transform="translate({marginLeft} 0)"
      stroke="currentColor"
      opacity="0.3"
    />

    <g bind:this={gHighValues} />
    <g bind:this={gLowValues} />
    <g bind:this={gCurrentPeriod} />

    <g bind:this={gx} transform="translate(0 {height - marginBottom})" />
    <g bind:this={gy} transform="translate({marginLeft} 0)" />
  </svg>

  <div class="flex justify-center gap-6 pb-2">
    <div class="flex items-center gap-2">
      <div class="w-4 h-4 consumers rounded-sm"></div>
      <span class="text-sm">Clients</span>
    </div>
    <div class="flex items-center gap-2">
      <div class="w-4 h-4 renewables rounded-sm"></div>
      <span class="text-sm">Renouvelables</span>
    </div>
  </div>
</div>

<style>
  .consumers {
    fill: var(--consumers-background-color);
    background-color: var(--consumers-background-color);
  }
  .renewables {
    fill: var(--renewable-background-color);
    background-color: var(--renewable-background-color);
    border-top-width: 2;
    border-top-color: black;
  }

  :global(.consumers-forecast) {
    fill: var(--consumers-background-color-faded);
    background-color: var(--consumers-background-color-faded);
  }
  :global(.renewables-forecast) {
    fill: var(--renewable-background-color-faded);
    background-color: var(--renewable-background-color-faded);
  }
</style>
