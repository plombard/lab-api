# --- Étape de compilation ---
FROM rust:1.93-slim-bookworm AS builder
WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN mkdir src
COPY src ./src/
COPY Cargo.lock .
COPY Cargo.toml .
RUN cargo build --release

# --- Étape d'exécution (Image finale légère) ---
FROM debian:bookworm-20260112-slim
WORKDIR /usr/local/bin
# On ne copie que le binaire
COPY --from=builder /usr/src/app/target/release/lab-api .
EXPOSE 8080
CMD ["./lab-api"]
