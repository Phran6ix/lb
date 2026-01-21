# lb-http-parser

A lightweight HTTP/1.x request parser written in Rust. This hobby academic project explores the internals of HTTP request parsing according to RFC 9112 specifications.

## Overview

`lb-http-parser` is a TCP server that listens for incoming HTTP requests and parses them into structured components. It supports the standard HTTP methods (GET, POST, PATCH, DELETE, PUT) and handles both fixed-length request bodies (via `Content-Length`) and chunked transfer encoding (via `Transfer-Encoding: chunked`).

## Architecture

The parser follows a state-machine approach with three main stages:

1. **Request Line Parser** (`src/internal/request.rs`)
   - Extracts HTTP method, request target, and HTTP version
   - Validates method support and HTTP specification compliance

2. **Headers Parser** (`src/internal/headers.rs`)
   - Parses field lines into a case-insensitive header map
   - Validates field name tokens according to RFC 9112

3. **Body Parser** (`src/internal/body/`)
   - Handles fixed-length bodies via `Content-Length` header
   - Supports chunked transfer encoding parsing
   - Enforces maximum message size constraints

The TCP listener (`src/cmd/tcplistener.rs`) binds to `127.0.0.1:8080` and delegates each incoming connection to the request parser.

## Building

Requires Rust toolchain. Build with:

```bash
cargo build
```

## Running

Start the server with:

```bash
cargo run
```

The server will listen on `127.0.0.1:8080` and log parsed HTTP requests to stdout.

## Testing

Run the test suite with:

```bash
cargo test
```

Tests are embedded in each parsing module and cover request line validation, header parsing, and body extraction scenarios.

## Status

**In Development** — This is a work-in-progress project. Some error handling paths contain placeholder implementations (`todo!()` macros).
