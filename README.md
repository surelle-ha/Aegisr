# Aegisr KV Database

Aegisr KVD is a next-generation in-memory data store designed to overcome the limitations of existing systems like Redis, Memcached, and others. It aims to provide **high performance, rich data structures, persistence options, clustering, and pub/sub messaging** while remaining simple to deploy and operate.

---

## Features

- High-performance in-memory key-value store
- Optional persistence for durability
- Rich data structures: lists, sets, sorted sets, hashes, streams, hyperloglogs
- Native pub/sub and event bus
- Multi-threaded core for better CPU utilization
- Auto-sharding and clustering with high availability
- Built-in metrics, monitoring, and recovery tools
- Modern SDKs for multiple languages

---

## Redis Alternatives and Opportunities for Aegisr

| **System** | **Type / Focus** | **Key Strengths** | **Key Flaws / Downsides** | **How Aegisr Can Improve** |
|------------|-----------------|-----------------|---------------------------|---------------------------|
| **Redis** | In-memory KV store, caching, pub/sub | Fast, rich data structures, persistence options, clustering | Single-threaded (limits CPU usage), clustering can be complex, memory-only mode is volatile | Multi-threaded core, simpler clustering, hybrid memory/disk storage |
| **Memcached** | Simple in-memory cache | Extremely fast, easy to deploy | No persistence, only strings, no replication, no pub/sub, simple LRU eviction | Add persistence, rich data structures, native pub/sub, intelligent eviction policies |
| **Hazelcast** | In-memory data grid | Distributed, supports maps/queues/topics, HA | Serialization overhead, slower than Redis for microsecond ops, complex scaling, enterprise features paid | Low-latency ops, free full-featured clustering, optimized serialization |
| **Aerospike** | Key-value store, low-latency, persistent | Microsecond reads/writes, auto-sharding | Limited data structures, operational complexity, smaller ecosystem, paid enterprise features | Rich data types, simplified deployment, open ecosystem |
| **etcd** | Strongly-consistent KV store | Strong consistency (Raft), config & service discovery | Disk-bound performance, no advanced data structures, not designed for high-throughput caching | Add high-throughput caching mode, richer structures, optional in-memory mode |
| **KeyDB** | Redis-compatible fork | Multi-threaded, active-active replication | Smaller community, edge-case Redis compatibility, complex threading bugs | Full Redis compatibility, robust multi-threading, better observability |
| **RocksDB / LevelDB** | Embedded KV stores | Persistent, fast on-disk storage | No networking, clustering, or pub/sub, memory operations limited | Build native networking, clustering, and in-memory acceleration |

---

## Key Themes for Aegisr to Address

1. **Performance vs persistence tradeoff:** Offer hybrid modes—fully in-memory for speed, optional persistence for durability.  
2. **Clustering simplicity:** Auto-sharding, auto-healing, multi-node replication with minimal ops overhead.  
3. **Data structure richness:** Lists, sets, sorted sets, hashes, bloom filters, hyperloglogs, streams.  
4. **Pub/Sub / messaging:** Built-in, low-latency messaging and event bus.  
5. **Operational simplicity:** Built-in metrics, monitoring, dashboards, and recovery tools.  
6. **Ecosystem & compatibility:** Modern client SDKs, strong documentation, optional Redis protocol support.

---

## Getting Started

> This section is a placeholder for future setup instructions.

## Launching the Aegisr Daemon

The Aegisr daemon must be running to operate the terminal interface. Use one of the commands below to start the daemon:

```bash
# Start the daemon on the default address 127.0.0.1:1211
./aegisr-daemon

# Bind the daemon to a custom host and port
./aegisr-daemon -H 0.0.0.0 -p 9090

# Start the daemon using a JSON configuration file
./aegisr-daemon -c <path_to_config_file>
```

### Example Configuration File

Create a JSON file to specify the host and port:

```json
{
  "host": "0.0.0.0",
  "port": 9000
}
```

> (1) Command-line options (-H for host, -p for port) override settings in the configuration file.
> (2) If neither command-line options nor a config file are provided, the daemon defaults to listening on 127.0.0.1:1211.
> (3) Ensure the specified host and port are available and not blocked by a firewall.

# Historical Performance Benchmarks

This document tracks the evolution of Aegisr's performance across versions. Metrics are collected using standard benchmarking tests for `put_value` and `get_value` operations.

## Benchmark Table

| **Version** | **Date** | **Put Ops/sec** | **Get Ops/sec** | **Put Latency (µs)** | **Get Latency (µs)** | **Notes** |
|-------------|----------|----------------|----------------|--------------------|--------------------|-----------|
| 1.0.1-beta  | 2025-11-17 | ~7,000          | ~100,000        | 136–145               | 9–10                  | Initial benchmark with 100 measurements. Found some outliers (7% put, 8% get). |
| 1.0.2-beta  | 2025-11-18 | ~122,000        | ~127,000        | 4.56–5.06             | 4.11–4.17             | Huge performance jump due to removal of autosave from every put/get. Outliers: 7–8% mild/high. |

## How to Update

1. Run benchmarks using the standard `put_value` and `get_value` tests.
2. Record average ops/sec for both operations.
3. Record latency range and any significant outliers.
4. Update the table with the new version, date, and notes on changes or optimizations.
5. Keep the table chronological for easy comparison of improvements over time.

## Notes

- Outliers in measurements are expected; record them in the notes column if significant.
- Include environment details if testing conditions change (CPU, memory, OS version, etc.).
- Consider adding visualizations in the future to track trends.
- This document serves as a historical record of Aegisr's performance evolution.
