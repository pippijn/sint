# Stage 1: Build Recipe (Caching)
FROM rust:1.91.1-slim-bullseye AS planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cache Dependencies (Server & Client)
FROM rust:1.91.1-slim-bullseye AS cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 3: Build Server
FROM rust:1.91.1-slim-bullseye AS builder-server
WORKDIR /app
COPY . .
# Copy cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
# Build Server
RUN cargo build --release --bin sint-server

# Stage 4: Build Client (WASM)
FROM rust:1.91.1-slim-bullseye AS builder-client
WORKDIR /app
# Install Trunk and WASM target
RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown
COPY . .
# Copy cached dependencies (Optimization: Might differ slightly for WASM but saves time)
# COPY --from=cacher /app/target target 
# We skip copying target for client because target arch is different (wasm32 vs x86_64)
# Build Client
WORKDIR /app/client
RUN trunk build --release

# Stage 5: Final Runtime (Nginx + Server)
FROM nginx:stable-alpine
WORKDIR /app

# Install dependencies if needed (glibc for rust binary if not fully static?)
# Rust slim-bullseye is glibc. Alpine is musl. 
# We should probably use a debian-slim base for runtime OR build with musl.
# Easiest: Use debian base and install nginx, OR use nginx-debian.
# But standard nginx image is based on debian usually (nginx:stable is debian).
# nginx:alpine is alpine.
# Let's check nginx:stable.

# Using debian-based nginx to ensure glibc compatibility with the built server
FROM nginx:stable

# Copy Server Binary
COPY --from=builder-server /app/target/release/sint-server /usr/local/bin/sint-server

# Copy Client Assets
COPY --from=builder-client /app/client/dist /usr/share/nginx/html

# Copy Nginx Config
COPY docker/nginx/nginx.conf /etc/nginx/nginx.conf

# Copy Entrypoint
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Expose HTTP
EXPOSE 80

# Run
CMD ["/usr/local/bin/entrypoint.sh"]
