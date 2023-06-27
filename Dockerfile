FROM ubuntu

RUN apt-get update

RUN apt-get install -y \
    curl \
    clang \
    gcc \
    g++ \
    zlib1g-dev \
    libmpc-dev \
    libmpfr-dev \
    libgmp-dev \
    git \
    cmake \
    pkg-config \
    libssl-dev \
    build-essential

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s - -y

ENV PATH=/root/.cargo/bin:${PATH}

WORKDIR /opt

COPY Cargo.toml Cargo.toml

COPY Cargo.lock Cargo.lock

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release --locked

RUN rm -rf src

COPY src src

CMD cargo run --release --locked
