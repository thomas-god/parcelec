<script lang="ts">
import {
  Chart,
  ChartColor,
  ChartData,
  ChartDataSets,
  ChartOptions,
  ChartPoint,
  Point,
  Scriptable
} from "chart.js";
import { Line, Bar } from "vue-chartjs";
import { Component, Vue, Watch, Prop } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import { Bid } from "../store/bids";

const bids_module = namespace("bids");

const options: ChartOptions = {
  maintainAspectRatio: false,
  responsive: true,
  legend: {
    position: "bottom"
  },
  tooltips: {
    intersect: false,
    mode: "index",
    axis: "x",
    position: "average",
    filter: (item: Chart.ChartTooltipItem, data: ChartData): boolean => {
      return item.yLabel! !== 0 && item.index! > 0;
    },
    callbacks: {
      title: (item: Chart.ChartTooltipItem[], data: ChartData): string => {
        return "";
      },
      label: (item: Chart.ChartTooltipItem, data: ChartData): string => {
        return (
          data.datasets![item.datasetIndex!].label +
          ": " +
          item.yLabel!.toLocaleString("fr-FR") +
          " €"
        );
      },
      footer: (item: Chart.ChartTooltipItem[], data: ChartData): string => {
        return item.length > 0 && item[0].index! > 0
          ? `Total: ${Math.floor(
              item.reduce((sum, val) => sum + Number(val.yLabel), 0)
            ).toLocaleString("fr-FR")} €`
          : "";
      }
    }
  },
  scales: {
    xAxes: [
      {
        scaleLabel: { labelString: "Phase", display: true },
        ticks: {
          suggestedMin: 1
        }
      }
    ],
    yAxes: [
      {
        type: "linear",
        stacked: true,
        gridLines: {
          drawOnChartArea: true
        },
        scaleLabel: { labelString: "€", display: true },
        ticks: {
          autoSkip: true,
          maxTicksLimit: 8
        }
      }
    ]
  }
};

@Component({
  extends: Line
})
export default class MainDataFinancesGraph extends Vue {
  @Prop() n_phases!: number;
  @Prop() results_by_type!: {
    type: string;
    name: string;
    color: string;
    values: number[];
  }[];
  @Prop() current_phase!: number;
  public renderChart!: (chartData: ChartData, options?: ChartOptions) => void;
  options = options;

  get results_fmt(): ChartDataSets[] {
    return this.results_by_type.map(type => {
      return {
        label: type.name,
        stack: "details",
        borderColor: "rgba(0, 0, 0, 0)",
        backgroundColor: type.color,
        pointRadius: 0,
        borderJoinStyle: "round",
        steppedLine: "after",
        data: [type.values[0]].concat(type.values),
        order: 1
      };
    });
  }
  get labels(): string[] {
    const labels = [];
    for (let i = 0; i <= this.n_phases; i++) {
      labels.push(String(i + 1));
    }
    return labels;
  }

  plot(): void {
    this.options!.scales!.xAxes![0].ticks!.min = 1;
    this.options!.scales!.xAxes![0].ticks!.max = this.n_phases;
    this.options!.scales!.xAxes![0].ticks!.maxTicksLimit = this.n_phases + 1;
    this.renderChart(
      {
        datasets: this.results_fmt,
        labels: this.labels
      },
      this.options
    );
  }

  mounted() {
    this.plot();
  }
  @Watch("results_fmt")
  watchConso(): void {
    this.plot();
  }
}
</script>