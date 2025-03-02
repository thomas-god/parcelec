## Mission statement for the MVP

Date: 03/02/2025

### Mission statement

Parcelec's MVP should focus on a simple experience that can be quickly picked by
new players, and yet that demonstrates some key concepts of electricity systems
and markets. In that regards it's ok to simplify some behaviors that would
occurred in real life, if its facilitates gameplay.

### Market design used

Thus we choose to focus the MVP entirely on the intraday-like market, as it's a
real time market and will more easily lead to interaction between the players.

We will still have a notion of settlement post-market, to make sure the players
have an incentive to balance their stack/portfolio.

A typical game should consist of several delivery periods during which players
can post offers and bids (orders), trade electricity, and decide how to dispatch
their power plants. Each delivery period should last a few minutes at most, in
order to give players a sense of urgency.

### Power plant technologies

We will limit the available power generation technologies to a few but distinct
ones. For instance :

- nuclear: cheap, but cannot stop/start immediately, nor change its power output
  too often,
- gas: expensive, but no other constraints,
- wind/solar: free, but cannot be controlled, and power output vary from period
  to period,
- storage/hydro: limited capacity, but can be used to store energy for later
  use.

### Tech stack

- Backend: Rust with tokio. No database/persistence layer, all state is kept in
  memory.
- Frontend: Svelte with Typescript, mobile first design (probably landscape
  still). Communication with the backend is done via websockets.
- Hosting: Docker stack in a VPS + CI/CD using Github Actions.
