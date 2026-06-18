# lb: A Simple Load Balancer in Rust

This project is an educational exploration into building a simple, asynchronous load balancer from scratch in Rust. What began as an academic exercise in HTTP/1.x parsing has evolved into a minimalist web framework that now serves as the foundation for the load balancer.

## Core Components (The Mini-Framework)

The project is built on a few core, custom-built components:

1.  **TCP Listener (`src/cmd/tcplistener.rs`)**: A basic TCP server that accepts incoming connections.
2.  **HTTP/1.x Parser (`src/internal/`)**: A state-machine parser built to comply with RFC 9112. It handles request lines, headers, and body parsing (including fixed-length and chunked encodings).
3.  **Router (`src/framework/`)**: A routing engine built on a custom **Radix Trie** implementation. It supports dynamic path segments (e.g., `/users/:id`) and query parameters.

## Project Status & Roadmap

The foundational framework components are complete. The project is now moving into the load balancer implementation phase.

### Completed
- [x] HTTP/1.x Request Parser
- [x] Radix Trie-based Router
- [x] Basic TCP Server

### Next Steps

1.  **Integrate Router & Listener**: Connect the router to the TCP listener to dispatch requests to handlers.
2.  **Implement Reverse Proxy**: Create a handler that forwards requests to an upstream server.
3.  **Manage Upstream Pool**: Design a structure to hold and manage a list of backend servers.
4.  **Implement Load Balancing Strategy**: Add a Round Robin strategy to select a server from the pool.

## Building

Requires the Rust toolchain.

```bash
cargo build
```

## Running

Start the server:

```bash
cargo run
```

The server will listen on `127.0.0.1:8080`.

## Testing

Run the test suite:

```bash
cargo test
```

