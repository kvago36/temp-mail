# -------------------
# 1. Билд Rust-бинарников внутри Linux
# -------------------
FROM rustlang/rust:nightly as builder

WORKDIR /app

# Копируем весь проект в контейнер
COPY . .

# Собираем оба бинарника в release для Linux
RUN cargo build --release

# -------------------
# 2. Образ для server
# -------------------
FROM ubuntu:latest as server
WORKDIR /app

# Копируем готовый бинарник из билдера
COPY --from=builder /app/target/release/server /app/server

# Устанавливаем системные зависимости (если нужны TLS, DB и т.д.)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

CMD ["/app/server"]

# -------------------
# 3. Образ для client
# -------------------
FROM ubuntu:latest as client
WORKDIR /app
COPY --from=builder /app/target/release/client /app/client
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

CMD ["/app/client"]
