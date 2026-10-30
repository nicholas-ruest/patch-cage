FROM rust:1.99-bookworm AS backend
WORKDIR /app
COPY . .
RUN cargo build --locked --release -p pc-api
FROM node:20-bookworm AS frontend
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend .
RUN npm run build
FROM debian:bookworm-slim
COPY --from=backend /app/target/release/pc-api /usr/local/bin/pc-api
COPY --from=frontend /app/frontend/dist /srv/ui
EXPOSE 8080
CMD ["pc-api"]
