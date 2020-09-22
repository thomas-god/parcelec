<template>
  <div id="nav">
    <router-link to="/">Accueil ⚡️</router-link>
    |
    <router-link to="/join">Rejoindre une partie</router-link>
    |
    <router-link to="/create">Créer une partie</router-link>
  </div>
</template>

<style scoped>
#nav {
  width: 100%;
  height: 34px;
  background-color: rgb(204, 218, 250);
  border-bottom: 1px solid black;
  position: -webkit-sticky;
  position: sticky;
  top: 0;
  display: flex;
  align-items: center;
  z-index: 10;
}
a {
  margin: auto 1rem;
  align-self: center;
  text-decoration: none;
  color: black;
  font-size: 1.2rem;
}
</style>
