# Natsume Backend

**Backend API for Natsume, a service within the Alcaris Network.**

Natsume is a core component of the Alcaris Network, responsible for managing item definitions, player data, and in-game economy logic. This backend provides RESTful APIs and internal logic to support game server functionality.

## Features

- Built with [Axum](https://github.com/tokio-rs/axum) and [Tokio](https://tokio.rs/)
- Designed for integration with PostgreSQL databases
- JSON serialization via Serde for clean API responses
- Ready for Kubernetes or containerized deployments

## Technologies

- **Language:** Rust
- **Framework:** Axum
- **Async Runtime:** Tokio
- **Database:** PostgreSQL
- **Serialization:** Serde (for JSON)

## Getting Started

```bash
# Build
make build

# Run (development)
make run
```

> Requires Rust >= 1.72 and a PostgreSQL database.

## License

This project is licensed under the MIT License, see the [LICENSE](LICENSE) file for details