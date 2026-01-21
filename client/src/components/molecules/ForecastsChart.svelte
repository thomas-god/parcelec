<script lang="ts">
  import * as d3 from "d3";

  import { type Forecasts } from "../organisms/Forecasts.svelte";

  let {
    height,
    width,
    total_forecasts,
    history,
  }: {
    width: number;
    height: number;
    total_forecasts: Forecasts;
    history: number[];
  } = $props();

  let marginTop = 20;
  let marginRight = 20;
  let marginBottom = 20;
  let marginLeft = 50;

  type PointType = "realized" | "active" | "forecast";

  let gx: SVGGElement;
  let gy: SVGGElement;
  let gyGrid: SVGGElement;
  let gBars: SVGGElement;
  let gErrorBars: SVGGElement;
  let svgElement: SVGElement;

  let data = $derived.by(() => {
    const points: {
      type: PointType;
      period: number;
      value: number;
      deviation: number;
    }[] = [];

    // Add history
    for (const [idx, value] of history.entries()) {
      points.push({
        type: idx + 1 < history.length ? "realized" : "active",
        period: idx + 1,
        value,
        deviation: 0,
      });
    }

    // Add forecasts
    for (const [key, value] of total_forecasts.entries()) {
      points.push({
        type: "forecast",
        period: key,
        value: value.value,
        deviation: value.deviation,
      });
    }
    return points;
  });

  let next_forecast = $derived.by(() => {
    for (const point of data) {
      if (point.deviation > 0) {
        return point;
      }
    }
  });

  let x = $derived(
    d3
      .scaleBand()
      .domain(data.map((point) => point.period.toString()))
      .range([marginLeft, width - marginRight])
      .padding(0.4),
  );

  let y = $derived(
    d3
      .scaleLinear()
      .domain([
        0,
        d3.max(data, (point) =>
          Math.abs(point.value - point.deviation),
        ) as number,
      ])
      .range([height - marginBottom, marginTop]),
  );

  const drawErrorBar = (context: d3.Path, point: (typeof data)[number]) => {
    // Don't draw error bar if no deviation
    if (point.deviation === 0) {
      return context;
    }
    const errorWidth = x.bandwidth() / 4;

    const xMiddle = (x(point.period.toString()) as number) + x.bandwidth() / 2;
    const yBottom = y(Math.abs(point.value + point.deviation));
    const yTop = y(Math.abs(point.value - point.deviation));

    // Bottom horizontal line
    context.moveTo(xMiddle - errorWidth, yBottom);
    context.lineTo(xMiddle + errorWidth, yBottom);

    // Middle vertical line
    context.moveTo(xMiddle, yBottom);
    context.lineTo(xMiddle, yTop);

    // Top horizontal line
    context.moveTo(xMiddle - errorWidth, yTop);
    context.lineTo(xMiddle + errorWidth, yTop);

    return context;
  };

  const barColor = (type: PointType): string => {
    if (type === "forecast") {
      return "steelblue";
    }
    if (type === "realized") {
      return "seagreen";
    }
    return "green";
  };

  const barStroke = (type: PointType): string => {
    if (type === "forecast") {
      return "darkblue";
    }
    return "limegreen";
  };

  const barStrokeWidth = (type: PointType): number => {
    if (type === "realized") {
      return 0;
    }
    if (type === "active") {
      return 1;
    }
    return 1;
  };

  const barOpacity = (type: PointType): number => {
    if (type === "realized") {
      return 1;
    }
    return 1;
  };

  $effect(() => {
    // Draw x axis
    d3.select(gx).call((sel) => sel.call(d3.axisBottom(x)));

    // Draw y axis
    d3.select(gy).call((sel) => sel.call(d3.axisLeft(y)));

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

    // Draw forecast rectangles
    d3.select(gBars).call((sel) =>
      sel
        .selectAll("rect")
        .data(data)
        .join("rect")
        .attr("x", (point) => x(point.period.toString()) as number)
        .attr("y", (point) => y(Math.abs(point.value)) as number)
        .attr("height", (point) => y(0) - y(Math.abs(point.value)))
        .attr("width", x.bandwidth())
        .attr("fill", (point) => barColor(point.type))
        .attr("stroke", (point) => barStroke(point.type))
        .attr("stroke-width", (point) => barStrokeWidth(point.type))
        .attr("opacity", (point) => barOpacity(point.type)),
    );

    // Draw forecast error bars
    d3.select(gErrorBars).call((sel) => {
      sel
        .selectAll("path")
        .data(data)
        .join("path")
        .attr("d", (point) => drawErrorBar(d3.path(), point).toString())
        .attr("stroke", "red")
        .attr("stroke-width", 1.5);
    });
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
