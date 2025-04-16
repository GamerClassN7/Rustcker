# ---- Build stage ----
    FROM rust:latest as builder

    WORKDIR /app
    COPY . .
    
    # Ensure dependencies are cached
    RUN cargo fetch 
    RUN cargo build --release
    
    # ---- Runtime stage ----
    FROM debian:bullseye-slim
    
    # Install minimal runtime dependencies
    RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
    
    COPY --from=builder /app/target/release/reverse-proxy /usr/local/bin/reverse-proxy
    COPY config.yaml /config.yaml
    
    # Default command
    CMD ["reverse-proxy"]
    