FROM ubuntu:24.04 AS build

RUN apt-get update && apt-get -y upgrade
RUN apt-get -y install \
    gcc \
    curl \
    pkg-config \
    libssl-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

COPY . /cln-backup

WORKDIR /cln-backup
RUN cargo build --release

FROM scratch AS binaries

COPY --from=build /cln-backup/target/release/backup /
