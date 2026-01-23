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
  let marginLeft = 50;

  type PointType = "realized" | "active" | "forecast";
  type Source = "consumers" | "renewables";

  let gx: SVGGElement;
  let gy: SVGGElement;
  let gyGrid: SVGGElement;
  let gBars: SVGGElement;
  let gErrorBars: SVGGElement;
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
  $inspect(width, fx("1"));
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

  let next_forecast = $derived.by(() => {
    for (const point of data) {
      if (point.deviation > 0) {
        return point;
      }
    }
  });

  $effect(() => {
    // Draw x axis
    d3.select(gx).call((sel) =>
      sel
        .call(d3.axisBottom(fx))
        .attr("transform", `translate(0,${height - marginBottom})`),
    );

    // Draw y axis
    d3.select(gy).call((sel) =>
      sel.call(d3.axisLeft(y)).attr("transform", `translate(${marginLeft},0)`),
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

    // Draw plain rectangle
    d3.select(gBars).call((sel) =>
      sel
        .selectAll("g")
        .data(d3.group(data, (elem) => elem.period))
        .join(
          (enter) =>
            enter
              .append("g")
              .attr(
                "transform",
                ([period]) => `translate(${fx(period.toString())},0)`,
              ),
          (update) =>
            update.attr(
              "transform",
              ([period]) => `translate(${fx(period.toString())},0)`,
            ),
          (exit) => exit.remove(),
        )
        .selectAll("rect")
        .data(([_period, points]) => points)
        .join("rect")
        .join(
          (enter) =>
            enter
              .append("rect")
              .attr("x", (point) => x(point.source) as number)
              .attr("y", (point) => y(point.value - point.deviation))
              .attr("width", x.bandwidth())
              .attr(
                "height",
                (point) => y(0) - y(point.value - point.deviation),
              )
              .attr("class", (point) => point.source),
          (update) =>
            update
              .attr("x", (point) => x(point.source) as number)
              .attr("y", (point) => y(point.value - point.deviation))
              .attr("width", x.bandwidth())
              .attr(
                "height",
                (point) => y(0) - y(point.value - point.deviation),
              )
              .attr("class", (point) => point.source),
        ),
    );

    d3.select(gErrorBars).call((sel) =>
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
        .attr("class", (point) => point.source)
        .attr("opacity", 0.5),
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

    <g bind:this={gBars} />
    <g bind:this={gErrorBars} />

    <g bind:this={gx} transform="translate(0 {height - marginBottom})" />
    <g bind:this={gy} transform="translate({marginLeft} 0)" />
  </svg>

  {#if next_forecast !== undefined}
    <div class="text-center italic pb-4">
      Prévision pour la prochaine période : <br />
      <span class="font-semibold">
        {Math.abs(next_forecast.value).toLocaleString("fr-FR")} ±
        {next_forecast.deviation.toLocaleString("fr-FR")} MW
      </span>
    </div>
  {/if}
</div>
