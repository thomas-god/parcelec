<template>
  <div class="home">
    <h1>Bienvenue sur Parcélec ! ⚡️</h1>
    <p>
      Parcélec est un jeu éducatif qui vous met dans la peau d'un gestionnaire
      de parc de production d'électricité. Votre but : fournir de l'énergie à
      vos clients en utilisant le plus efficacement possible les centrales en
      votre possession et en ayant recours si besoin au marché de l'électricité.
    </p>
    <h2>Déroulé d'une partie</h2>
    <p>
      Une partie se compose d'une succession de phases dans lesquelles vous
      serez confronté·e à différents niveaux de consommation.
    </p>
    <p>
      Au début de chaque phase vos équipes commerciales vous indiqueront quel
      niveau de consommation vous devrez couvrir (attention tout MWh non couvert
      vous expose à une pénalité financière !).
    </p>
    <p>
      Il vous faudra alors régler les points de consigne de vos centrales et
      acheter ou vendre de l'énergie à d'autres producteurs pour répondre au
      mieux à ce besoin. À la fin d'une phase vous recevrez un bilan de
      performances énergétiques et financières.
    </p>
    <h2>Votre portefeuille</h2>
    <div class="grid__left">
      <power-plants-list class="card" :dummy="true" />
      <div>
        <p>
          Pour mener à bien votre mission vous disposez de plusieurs centrales
          aux caractéristiques variées. La carte d'identité de chaque centrale
          vous indique ses puissances maximale et minimale (une centrale ne peut
          produire dans la bande grisée), son coût à produire de l'énergie, et
          son éventuel stock (à utiliser donc avec parcimonie!).
        </p>
        <p>
          Il existe plusieurs types de centrales : les centrales nucléaires ☢️
          ont un coût de production faible et des puissances maximales
          importantes mais ne peuvent fonctionner à faibles puissances, les
          centrales thermiques 🔥 coûtent cher à produire mais sont très
          flexibles, les centrales hydrauliques 💧 ne coutent rien à produire
          mais on un stock limité, les centrales de stockage 🔋 peuvent être
          rechargées si leur stock est trop bas, et enfin les centrales
          renouvelables ☀️ ne coûtent rien à produire mais vous ne pouvez pas
          choisir leur point de fonctionnement.
        </p>
        <p>
          Pour régler le niveau de puissance d'une centrale déplacez son curseur
          au-delà de la bande grisée. Vous pouvez voir votre production totale
          en temps réel (actuellement
          {{ prod_mw }} MW). Entraînez vous !
        </p>
        <p>
          Une fois qu'un plan de production vous convient vous devez l'envoyer
          au gestionnaire du réseau en cliquant sur le bouton <em>Envoyer</em>.
          Vous pouvez modifier votre plan de production autant de fois que vous
          le souhaitez avant la fin d'une phase (mais n'oubliez pas de
          l'envoyer!). Le bouton <em>Effacer</em> vous permet de revenir à votre
          précédent plan de production.
        </p>
      </div>
    </div>
    <h2>Le marché</h2>
    <div class="grid__right">
      <div>
        <p>
          Il se peut qu'à certains moments vos centrales ne suffisent pas pour
          couvrir toute votre consommation : vous pourrez alors essayer
          d'acheter l'énergie qu'il vous manque sur le marché. Le marché est un
          endroit où les différents acteurs du jeu peuvent poster des offres
          d'achat ou de vente d'énergie.
        </p>
        <p>
          Attention, poster une offre ne suffit pas à vous garantir l'achat ou
          la vente du volume d'énergie : pour que l'échange se fasse il faut
          qu'il y ait un vendeur (ou un acheteur) disposé à faire l'échange à un
          prix compatible au votre !
        </p>
        <p>
          Une fois les enchères fermées, vous saurez si vos offres ont été
          retenues (il se peut que seule une partie soit retenue), et il vous
          faudra ajuster votre planning de production en conséquence !
        </p>
      </div>
      <bids-list class="card" :dummy="true" />
    </div>

    <button class="home__create_game" @click="$router.push('/create')">
      Commencer une partie
    </button>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from "vue-property-decorator";
import { State, Action, Getter, namespace } from "vuex-class";
import PowerPlantsList from "../components/PowerPlantsList.vue";
import BidsList from "../components/BidsList.vue";
import { PowerPlant } from "../store/portfolio";

const portfolioModule = namespace("portfolio");

@Component({ components: { PowerPlantsList, BidsList } })
export default class Home extends Vue {
  @portfolioModule.Mutation SET_POWER_PLANTS!: (
    power_plants: PowerPlant[]
  ) => void;
  @portfolioModule.State power_plants!: PowerPlant[];

  get prod_mw() {
    return this.power_plants.reduce(
      (a, b) => a + Number(b.planning_modif),
      0 as number
    );
  }

  created() {
    // Set dummy portfolio for demonstration purpose
    this.SET_POWER_PLANTS([
      {
        id: "1",
        type: "nuc",
        p_min_mw: 500,
        p_max_mw: 1300,
        stock_max_mwh: -1,
        price_eur_per_mwh: 25,
        planning: 0,
        planning_modif: 0,
        stock_mwh: -1
      },
      {
        id: "2",
        type: "hydro",
        p_min_mw: 50,
        p_max_mw: 500,
        stock_max_mwh: 1000,
        price_eur_per_mwh: 0,
        planning: 0,
        planning_modif: 0,
        stock_mwh: 1000
      },
      {
        id: "3",
        type: "therm",
        p_min_mw: 150,
        p_max_mw: 600,
        stock_max_mwh: -1,
        price_eur_per_mwh: 65,
        planning: 0,
        planning_modif: 0,
        stock_mwh: -1
      },
      {
        id: "4",
        type: "ren",
        p_min_mw: 0,
        p_max_mw: 350,
        stock_max_mwh: -1,
        price_eur_per_mwh: 0,
        planning: 150,
        planning_modif: 0,
        stock_mwh: -1
      },
      {
        id: "5",
        type: "storage",
        p_min_mw: 100,
        p_max_mw: 500,
        stock_max_mwh: 1000,
        price_eur_per_mwh: 0,
        planning: 0,
        planning_modif: 0,
        stock_mwh: 500
      }
    ]);
  }
}
</script>

<style scoped>
.home {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.home p {
  font-size: 1.2rem;
  text-align: start;
  margin: 0 0rem 1rem;
  word-break: break-word;
  hyphens: auto;
}

.home p {
  max-width: 600px;
}

.card {
  padding: 1rem;
  border: 1px solid rgba(0, 0, 0, 0.493);
  border-radius: 3px;
  width: 100%;
  max-width: 500px;
  box-sizing: border-box;
}

@media screen and (min-width: 900px) {
  .home {
    padding: 0 1.5rem;
  }
  .grid__left {
    display: grid;
    grid-template-columns: 1fr 1fr;
    align-items: center;
    gap: 2rem;
  }
  .grid__left .card {
    justify-self: end;
  }

  .grid__right {
    display: grid;
    grid-template-columns: 1fr 1fr;
    align-items: center;
    gap: 2rem;
  }
  .grid__right .card {
    justify-self: start;
  }
  .grid__right p {
    text-align: end;
  }
}
@media screen and (max-width: 900px) {
  .grid__left {
    display: grid;
    grid-template-rows: auto auto;
    justify-items: center;
    align-items: flex-start;
  }
  .grid__left p,
  .grid__right p,
  .home p {
    text-align: initial;
    padding: 0 1.5rem;
  }

  .grid__right {
    display: grid;
    grid-template-rows: auto auto;
    justify-items: center;
  }
  .grid__right > .card {
    grid-row-start: 1;
  }
  .grid__left .card,
  .grid__right .card {
    margin-bottom: 1rem;
  }
}

@media screen and (max-width: 520px) {
  .card {
    border: none;
    position: relative;
  }
  .card::before,
  .card::after {
    content: "";
    position: absolute;
    left: 12.5%;
    width: 75%;
    height: 1px;
    border-bottom: 2px solid gray;
  }
  .card::before {
    top: 0;
  }
  .card::after {
    bottom: 0;
  }
}

.home__create_game {
  border: none;
  border-radius: 1rem;
  background-color: rgb(0, 132, 255);
  margin: 1rem 1rem 3rem 1rem;
  padding: 5px 10px;
  font-size: 1.3rem;
  font-weight: normal;
  color: white;
}
</style>
