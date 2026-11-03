# [parcelec.org](https://parcelec.org)

[parcelec.org](https://parcelec.org) est un jeu éducatif qui vous place dans le rôle d'un producteur
d'électricité. Votre but ? Satisfaire la consommation de vos clients en utilisant astucieusement vos
centrales et les marchés de l'électricité.

## Running locally with Docker

The whole app is packaged as a single image with no extra dependencies:
[`ghcr.io/thomas-god/parcelec`](https://github.com/thomas-god/parcelec/pkgs/container/parcelec).

Pull the `main` tag and run it, exposing the API port (`9002` by default):

```sh
docker run --rm -p 9002:9002 \
  -e ALLOW_ORIGIN=http://localhost:9002 \
  ghcr.io/thomas-god/parcelec:main
```

The app is now available at <http://localhost:9002>.

To build the image yourself instead, run from the repository root:

```sh
docker build -f infra/Dockerfile -t parcelec .
```

The app has no persistence layer so there are no volume to mount nor backup to setup.

## Running e2e tests

- The `client` must first be built before running the e2e tests. This also means you have to rebuild
  it if you change something on the `client`.

```sh
cd client
npm run build
```

- Then start `geckodriver` on port `4444`. You might need to run it as sudo depending on how you
  [installed it](https://github.com/mozilla/geckodriver?tab=readme-ov-file#installation).

```sh
geckodriver --port=4444
```

- Finally you can rust e2e tests from `app` (use env `DISPLAY=true` to toggle headless mode on and
  off for easier visual debugging):

```sh
cd app
DISPLAY=true cargo test --features e2e-tests
```

## Credits

Favicon icon from [OpenMoji](https://openmoji.org/) – the open-source emoji and icon project.
License: [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/)
