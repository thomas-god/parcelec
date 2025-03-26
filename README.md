# [parcelec.org](https://parcelec.org)

[parcelec.org](https://parcelec.org) est un jeu éducatif qui vous place dans le
rôle d'un producteur d'électricité. Votre but ? Satisfaire la consommation de
vos clients en utilisant astucieusement vos centrales et les marchés de
l'électricité.

## Running e2e tests

- The `client` must first be built before running the e2e tests. This also means
  you have to rebuild it if you change something on the `client`.

```sh
cd client
npm run build
```

- Then start `geckodriver` on port `4444`. You might need to run it as sudo
  depending on how you
  [installed it](https://github.com/mozilla/geckodriver?tab=readme-ov-file#installation).

```sh
geckodriver --port=4444
```

- Finally you can rust e2e tests from `app`:

```sh
cd app
cargo test --features e2e-tests
```
