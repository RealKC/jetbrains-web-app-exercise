FROM rust:latest

ENV DATABASE_URL sqlite:database.sqlite

WORKDIR /usr/src/blog
COPY . .

RUN cargo install sqlx-cli
RUN cargo sqlx db create
RUN cargo sqlx migrate run

RUN cargo install --path .

CMD ["jetbrains-web-app-exercise"]
