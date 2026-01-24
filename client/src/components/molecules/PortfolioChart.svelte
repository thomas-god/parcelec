<script lang="ts">
  import * as d3 from "d3";

  type PortfolioType =
    | "consumers"
    | "renewables"
    | "nuclear"
    | "gas"
    | "storage"
    | "market";

  export type PortfolioVolumes = {
    consumers: number;
    renewable: number;
    nuclear: number;
    gas: number;
    storage: number;
    marketSold: number;
    marketBought: number;
  };

  interface Props {
    height?: number;
    iconSize?: number;
    volumes: PortfolioVolumes;
  }

  let { height = 300, volumes, iconSize = 0.3 }: Props = $props();
  let chartWidth: number = $state(300);
  let svgElement: SVGElement;
  let gYaxis: SVGGElement;
  let gBars: SVGGElement;

  const margin = 8;

  const icons: Record<PortfolioType, string> = {
    consumers: "/icons/consumers.svg",
    gas: "/icons/gas.svg",
    market: "/icons/market.svg",
    nuclear: "/icons/nuclear.svg",
    renewables: "/icons/renewable.svg",
    storage: "/icons/storage.svg",
  };

  type Data = {
    sign: "positive" | "negative";
    type: PortfolioType;
    value: number;
  }[];

  let data: Data = $derived([
    {
      sign: "negative",
      type: "consumers",
      value: Math.abs(volumes.consumers),
    },
    { sign: "positive", type: "renewables", value: volumes.renewable },
    { sign: "positive", type: "nuclear", value: volumes.nuclear },
    { sign: "positive", type: "gas", value: volumes.gas },
    {
      sign: "negative",
      type: "storage",
      value: volumes.storage < 0 ? Math.abs(volumes.storage) : 0,
    },
    {
      sign: "positive",
      type: "storage",
      value: volumes.storage > 0 ? Math.abs(volumes.storage) : 0,
    },
    {
      sign: "negative",
      type: "market",
      value: Math.abs(volumes.marketSold),
    },
    {
      sign: "positive",
      type: "market",
      value: Math.abs(volumes.marketBought),
    },
  ]);

  let lastNegativeType: PortfolioType | undefined = $derived.by(() => {
    if (Math.abs(volumes.marketSold) > 0) {
      return "market";
    }
    if (volumes.storage < 0) {
      return "storage";
    }
    if (Math.abs(volumes.consumers) > 0) {
      return "consumers";
    }
    return undefined;
  });
  let lastPositiveType: PortfolioType | undefined = $derived.by(() => {
    if (Math.abs(volumes.marketBought) > 0) {
      return "market";
    }
    if (volumes.storage > 0) {
      return "storage";
    }
    if (volumes.gas > 0) {
      return "gas";
    }
    if (volumes.nuclear > 0) {
      return "nuclear";
    }
    if (volumes.renewable > 0) {
      return "renewables";
    }
    return undefined;
  });
  $inspect(lastNegativeType, lastPositiveType);

  let stackedData = $derived(
    d3
      .stack()
      .keys(d3.union(data.map((point) => point.type)))
      .value(([_sign, sign_values], type) => {
        return sign_values.get(type) === undefined
          ? 0
          : sign_values.get(type).value;
      })(
      d3.index<Data, unknown>(
        data,
        (d) => d.sign,
        (d) => d.type,
      ),
    ),
  );

  let x = $derived(
    d3
      .scaleLinear()
      .domain([
        0,
        Math.max(
          d3.max(stackedData, (d) => d3.max(d, (d) => d[1])),
          Math.abs(volumes.consumers * 1.1),
        ),
      ])
      .range([0, chartWidth - margin]),
  );

  let y = $derived(
    d3
      .scaleBand()
      .domain(
        d3.groupSort(
          data,
          (a, b) => (a[0].sign === "positive" ? 1 : -1),
          (d) => d.sign,
        ),
      )
      .range([margin, height - margin])
      .padding(0.2),
  );

  const drawRectanglePath = (point: any, noWidth = false) => {
    const x0 = x(point[0]);
    const y0 = y(point.data[0]);
    const width = noWidth ? 0 : x(point[1]) - x(point[0]);
    const height = y.bandwidth();

    let radius = 0;
    if (
      (point.sign === "positive" && point.key === lastPositiveType) ||
      (point.sign === "negative" && point.key === lastNegativeType)
    ) {
      radius = height / 3;
    }
    return `
    M${x0},${y0} h${width - radius}
    q${radius},0 ${radius},${radius}
    v${height - 2 * radius}
    q0,${radius} ${-radius},${radius}
    h-${width - radius} z`;
  };

  const drawRectanglePathNoWidth = (point: any) =>
    drawRectanglePath(point, true);

  $effect(() => {
    d3.select(gYaxis)
      .call(d3.axisLeft(y))
      .selectAll(".domain")
      .attr("stroke-width", 2);

    d3.select(gBars).call((sel) => {
      const groups = sel.selectAll("g").data(stackedData).join("g");

      // Create/update rectangles
      groups
        .selectAll("path")
        .data((series) => {
          // Each series is an array of elements, with a key = Setpoint
          // series.key -> consumers | market | ...
          // series[0] -> negative datapoints
          // series[1] -> positive datapoints
          // series[0].data[0] -> key "negative"
          // series[0].data[1] -> Map avec les setpoint en keys (consumers, market, etc.)
          // series[0].data[1].get("consumers") -> l'object inital des data

          const values = series.map((point) => {
            // Map extra information to individual points
            const value =
              point.data[1].get(series.key) === undefined
                ? 0
                : point.data[1].get(series.key).value;
            point.key = series.key;
            point.empty = value === 0;
            point.sign = point.data[0];
            return point;
          });
          // Skip empty datapoints
          return values.filter((value) => !value.empty);
        })
        .join(
          (enter) =>
            enter
              .append("path")
              .attr("d", (point) => drawRectanglePathNoWidth(point))
              .transition(d3.transition().duration(200).ease(d3.easeLinear))
              .attr("d", (point) => drawRectanglePath(point))
              .attr("class", (d) => d.key),

          (update) =>
            update
              .transition(d3.transition().duration(200).ease(d3.easeLinear))
              .attr("d", (point) => drawRectanglePath(point)),
          (exit) =>
            exit
              .transition(d3.transition().duration(200).ease(d3.easeLinear))
              .attr("d", (point) => drawRectanglePathNoWidth(point))
              .remove(),
        );

      // Create/update text labels (one per series)
      groups
        .selectAll("image")
        .data((series) => {
          const nonEmptyPoints = series.filter((point) => {
            const value =
              point.data[1].get(series.key) === undefined
                ? 0
                : point.data[1].get(series.key).value;
            return value !== 0;
          });
          return nonEmptyPoints.map((point) => ({
            key: series.key,
            point: point,
          }));
        })
        .join(
          (enter) =>
            enter
              .append("image")
              .attr("href", (d) => icons[d.key])
              .attr("width", y.bandwidth() * iconSize)
              .attr("height", y.bandwidth() * iconSize)
              .attr(
                "y",
                (d) =>
                  y(d.point.data[0]) + (y.bandwidth() * (1 - iconSize)) / 2,
              )
              .attr("x", (d) => x(d.point[0]) - y.bandwidth() * iconSize)
              .transition(d3.transition().duration(200).ease(d3.easeLinear))
              .attr(
                "x",
                (d) =>
                  (x(d.point[1]) + x(d.point[0])) / 2 -
                  (y.bandwidth() * iconSize) / 2,
              ),
          (update) =>
            update
              .transition(d3.transition().duration(200).ease(d3.easeLinear))
              .attr(
                "x",
                (d) =>
                  (x(d.point[1]) + x(d.point[0])) / 2 -
                  (y.bandwidth() * iconSize) / 2,
              )
              .attr(
                "y",
                (d) =>
                  y(d.point.data[0]) + (y.bandwidth() * (1 - iconSize)) / 2,
              )
              .attr("width", y.bandwidth() * iconSize)
              .attr("height", y.bandwidth() * iconSize),
          (exit) =>
            exit
              .transition(d3.transition().duration(200).ease(d3.easeLinear))
              .attr("opacity", 0)
              .remove(),
        );
    });
  });
</script>

<div class="w-full" bind:clientWidth={chartWidth}>
  <svg
    width={chartWidth}
    {height}
    viewBox={`0 0 ${chartWidth} ${height}`}
    role="img"
    class="h-full w-full select-none"
    bind:this={svgElement}
  >
    <g bind:this={gBars} />
    <g bind:this={gYaxis} />
  </svg>
</div>
