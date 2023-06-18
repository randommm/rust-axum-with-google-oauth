## About

You can use this repo as a template for OAuth authentication using Axum and Google OAuth.

The underling database used is MongoDB, but it should be relativelly straighforward to adapt the code to other databases.

Minijinja is also used as the HTML template system. Moreover, a deployment example with GitHub Actions is provided.

## Conventional setup

* Get an OAuth Client ID and key at https://console.cloud.google.com/apis/credentials, setup `http://localhost:3011/oauth_return` as an authorised redirect URI.

* Start a MongoDB instance, e.g. install mongo with your system package manager or with Docker `docker run --name mongo-server -p 27017:27017 -d mongo`

* Create file named `.env` at the root of the repository (same folder as the README.md), containing:

      DATABASE_URI="mongodb://localhost:27017/"
      GOOGLE_CLIENT_ID=your_google_oauth_id
      GOOGLE_CLIENT_SECRET=your_google_oauth_secret

* If you don't have `Rust` installed, see `https://rustup.rs`.

* Deploy with `cargo run --release`, then just browse your website at `http://localhost:3011`.

## Setup with Docker Compose

* Get an OAuth Client ID and key at https://console.cloud.google.com/apis/credentials, setup `http://localhost:3011/oauth_return` as an authorised redirect URI.

* Create file named `.env` at the root of the repository (same folder as the README.md), containing:

      DATABASE_URI="mongodb://mongo-server/27017/"
      GOOGLE_CLIENT_ID=your_google_oauth_id
      GOOGLE_CLIENT_SECRET=your_google_oauth_secret

* Build your OCI (Docker image) with `docker build -t ghcr.io/randommm/rust-axum-with-google-oauth .`.

* Deploy with `docker-compose up`, then just browse your website at `http://localhost:3011`.

## Setup with Docker (but without Docker Compose)

* Get an OAuth Client ID and key at https://console.cloud.google.com/apis/credentials, setup `http://localhost:3011/oauth_return` as an authorised redirect URI.

* Start a MongoDB instance: `docker run --name mongo-server -p 27017:27017 -d mongo`

* Build your OCI (Docker image) with `docker build -t ghcr.io/randommm/rust-axum-with-google-oauth .`.

* Deploy with `docker run --env DATABASE_URI="mongodb://127.0.0.1:27017/" --env GOOGLE_CLIENT_ID=your_google_oauth_id --env GOOGLE_CLIENT_SECRET=your_google_oauth_secret --rm -p 3011:3011 --net host ghcr.io/randommm/rust-axum-with-google-oauth`, then just browse your website at `http://localhost:3011`.
