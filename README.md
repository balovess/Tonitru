# Tonitru

Tonitru is an innovative open-source project that reimagines data exchange for the networked world. Designed for developers and architects building the next generation of distributed systems, Tonitru delivers a blazing-fast, secure, and future-proof data format that empowers seamless communication across cloud, edge, and everything in between.

## Why Tonitru?
- **Network-native DNA:** Purpose-built for modern, distributed, and heterogeneous environments.
- **Lightning Performance:** Ultra-efficient serialization and deserialization, ready for high-throughput and low-latency scenarios.
- **Rock-solid Security:** Integrated encryption and integrity checks keep your data safe in transit.
- **Effortless Evolution:** Flexible schema management supports forward and backward compatibility, so your systems can grow without friction.
- **Plug & Play Extensibility:** Modular design makes it easy to add new features, protocols, or custom logic.

## Who Should Use Tonitru?
Tonitru is ideal for anyone building:
- Cloud-native applications
- Edge computing solutions
- IoT platforms
- High-performance APIs
- Any system where data exchange speed, security, and adaptability matter

## Getting Started
Clone the repository and explore the documentation in the `docs/` directory to get up and running quickly.

```bash
# 启用SIMD优化进行构建
cargo build --features simd

# 运行测试（包括SIMD优化测试）
cargo test --features simd
```

## Key Features

### Four-Stage Pipeline Processor
Tonitru implements a high-performance four-stage pipeline for batch decoding:
1. **Prefetch**: Prepare data for efficient processing
2. **Decode**: Convert raw bytes to typed values
3. **Dispatch**: Process decoded values
4. **Verify**: Validate decoded data

### SIMD Optimizations
- Supports multiple instruction sets (SSE4.1, AVX2, AVX-512, NEON)
- Automatically selects the best available instruction set
- Safe memory management with `BatchResult` enum
- Zero-copy parsing for aligned data

### Compression Strategies
- Zstandard (zstd) compression
- Brotli compression
- Sharded compression for large data
- Incremental compression with context-aware dictionaries

### Schema System
- Comprehensive type system with cross-platform consistency
- Flexible default value strategies for nested objects
- JSON Schema-compatible parser with Tonitru extensions
- Automatic type inference from sample data
- Robust validation with configurable constraints

## Documentation
Comprehensive guides, design documents, and development plans are available in the `docs/` directory:
- [Four-Stage Pipeline Processor](docs/pipeline_processor.md)
- [Pipeline Processor Refactoring](docs/pipeline_processor_refactoring.md)
- [SIMD Optimizations](docs/simd_optimizations.md)
- [BatchResult Design](docs/batch_result.md)
- [Compression Module](docs/compression_module.md)
- [Schema Module](docs/schema_module.md)

## License
Tonitru is licensed under the Apache License 2.0. This means you are free to use, modify, and distribute the project, provided that you retain proper attribution and keep the project open source. For full details, see the LICENSE file in this repository.
