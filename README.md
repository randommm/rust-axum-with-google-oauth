## About

You can use this repo as a template for OAuth authentication using Axum and Google OAuth.

The underling database used is SQLite using SQLx, see branch the mongodb for a MongoDB version.

Minijinja is also used as the HTML template system. Moreover, a deployment example with GitHub Actions is provided.

## Live Demo

A live demo of this template is available at:

https://rust-oauth.marcoinacio.com/

## Conventional setup

* Get an OAuth Client ID and key at https://console.cloud.google.com/apis/credentials, setup `http://localhost:3011/oauth_return` as an authorised redirect URI.

* Create file named `.env` at the root of the repository (same folder as the README.md), containing:

      DATABASE_URI=sqlite://db/db.sqlite3
      GOOGLE_CLIENT_ID=your_google_oauth_id
      GOOGLE_CLIENT_SECRET=your_google_oauth_secret

* If you don't have `Rust` installed, see `https://rustup.rs`.

* Create the database: `cargo install sqlx-cli && sqlx database create && sqlx migrate run`.

* Deploy with `cargo run --release`, then just browse your website at `http://localhost:3011`.

## Setup with Docker Compose

* Get an OAuth Client ID and key at https://console.cloud.google.com/apis/credentials, setup `http://localhost:3011/oauth_return` as an authorised redirect URI.

* Create file named `.env` at the root of the repository (same folder as the README.md), containing:

      DATABASE_URI=sqlite://db/db.sqlite3
      GOOGLE_CLIENT_ID=your_google_oauth_id
      GOOGLE_CLIENT_SECRET=your_google_oauth_secret

* Build your OCI (Docker image) with `docker build -t ghcr.io/randommm/rust-axum-with-google-oauth .`.

* Deploy with `docker-compose up`, then just browse your website at `http://localhost:3011`.

## Setup with Docker (but without Docker Compose)

* Get an OAuth Client ID and key at https://console.cloud.google.com/apis/credentials, setup `http://localhost:3011/oauth_return` as an authorised redirect URI.

* Build your OCI (Docker image) with `docker build -t ghcr.io/randommm/rust-axum-with-google-oauth .`.

* Deploy with `docker run --env DATABASE_URI="sqlite://db/db.sqlite3" --env GOOGLE_CLIENT_ID=your_google_oauth_id --env GOOGLE_CLIENT_SECRET=your_google_oauth_secret --rm -p 3011:3011 -v db:db ghcr.io/randommm/rust-axum-with-google-oauth`, then just browse your website at `http://localhost:3011`.

## Optional extra: production deploy with Nginx

You can additional deploy on Nginx by adding the following to its configuration file (generally located at `/etc/nginx/sites-available/default`):

      server {
            server_name youdomainname.com;
            listen 80;
            location / {
                  proxy_pass http://127.0.0.1:3011;
                  proxy_set_header        Host $host;
            }
      }

Additionally, you can obtain an SSL certificate by running `certbot --nginx`

## Optional extra: automate deploy Github Actions

Generate a new token (classic) at https://github.com/settings/tokens/new with `read:packages` permission. Login to the GH Docker registry at your server with `docker login ghcr.io`.

Now, place `docker-compose.yml` on your server along with the `.env` and edit the `docker-compose.yml`, change:

      container_name: rust-axum-with-google-oauth
      build:
          context: .

to:

      container_name: rust-axum-with-google-oauth
      image:  ghcr.io/put_your_github_username_here/rust-axum-with-google-oauth

Finally, edit your crontab (`crontab -e`) to auto check, pull and deploy changes every 5 minutes:

      */5 * * * * cd /folder_where_docker-compose.yml_is_located/ && /usr/bin/docker-compose pull && /usr/bin/docker-compose up -d
