FROM rust:1.73-slim as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/type-ahead /usr/local/bin/myapp
EXPOSE 5050
CMD ["myapp"]