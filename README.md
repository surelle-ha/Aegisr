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

## Key Themes for Aegisr to Address

1. **Performance vs persistence tradeoff:** Offer hybrid modes—fully in-memory for speed, optional persistence for durability.  
2. **Clustering simplicity:** Auto-sharding, auto-healing, multi-node replication with minimal ops overhead.  
3. **Data structure richness:** Lists, sets, sorted sets, hashes, bloom filters, hyperloglogs, streams.  
4. **Pub/Sub / messaging:** Built-in, low-latency messaging and event bus.  
5. **Operational simplicity:** Built-in metrics, monitoring, dashboards, and recovery tools.  
6. **Ecosystem & compatibility:** Modern client SDKs, strong documentation, optional Redis protocol support.

---

# Getting Started

## Launching the Aegisr Daemon

The Aegisr daemon must be running to operate the terminal interface. Get the daemon and terminal from the [releases page](https://github.com/surelle-ha/Aegisr/releases).

Use one of the commands below to start the daemon:

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

## Running Aegisr with Docker

To run the Aegisr daemon using Docker, use the following command:

```bash
docker pull surelle/aegisr:latest
docker run -d -p 1211:1211 surelle/aegisr:latest
```

# Aegisr Terminal Commands

Once the Aegisr daemon is running, you can interact with it using the Aegisr terminal. Below are the available commands:

| **Command** | **Arguments** | **Description** |
|------------|---------------|-----------------|
| `init` | `--verbose`, `--reset` | Initialize configuration files. Optionally reset them. |
| `list` | *(none)* | List all collections. |
| `use <name>` | `--verbose` | Switch to and activate a specific collection. |
| `new <name>` | `--verbose` | Create a new collection. |
| `delete <name>` | `--verbose` | Delete an existing collection. |
| `rename <name> <new_name>` | `--verbose` | Rename a collection. |
| `status` | *(none)* | Show the current collection and daemon status. |
| `put <key> <value>` | `--verbose` | Store a key/value pair in the active collection. |
| `get <key>` | `--verbose` | Retrieve the value for a key in the active collection. |
| `del <key>` | `--verbose` | Delete a key/value pair from the active collection. |
| `clear` | `--verbose` | Clear all key/value entries in the active collection. |

## Command Schema 

Each command follows the schema:

```bash
aegisr <command> [arguments] [options]
```

### Sample Session

```bash
developer@aegisr-lab:~/aegisr$ ./aegisr init
{
  "message": "Engine initialized. Active Collection: default",                                                                                                                                                                                                     
  "status": "ok"                                                                                                                                                                                                                                                   
}                                                                                                                                                                                                                                                                  
developer@aegisr-lab:~/aegisr$ ./aegisr init
{
  "message": "Engine initialized. Active Collection: default",                                                                                                                                                                                                     
  "status": "ok"                                                                                                                                                                                                                                                   
}                                                                                                                                                                                                                                                                  
developer@aegisr-lab:~/aegisr$ ./aegisr list
{
  "data": [                                                                                                                                                                                                                                                        
    "default"                                                                                                                                                                                                                                                      
  ],                                                                                                                                                                                                                                                               
  "status": "ok"                                                                                                                                                                                                                                                   
}                                                                                                                                                                                                                                                                  
developer@aegisr-lab:~/aegisr$ ./aegisr new my_db
{
  "message": "✓ Collection 'my_db' created",                                                                                                                                                                                                                       
  "status": "ok"                                                                                                                                                                                                                                                   
}                                                                                                                                                                                                                                                                  
developer@aegisr-lab:~/aegisr$ ./aegisr use my_db
{
  "message": "Active Collection set to 'my_db'",                                                                                                                                                                                                                   
  "status": "ok"                                                                                                                                                                                                                                                   
}                                                                                                                                                                                                                                                                  
developer@aegisr-lab:~/aegisr$ ./aegisr put my_password HelloWorld123
{
  "message": "✓ Key 'my_password' saved in collection 'my_db' (in-memory)",                                                                                                                                                                                        
  "status": "ok"                                                                                                                                                                                                                                                   
}                                                                                                                                                                                                                                                                  
developer@aegisr-lab:~/aegisr$ ./aegisr get my_password
{
  "message": "HelloWorld123",                                                                                                                                                                                                                                      
  "status": "ok"                                                                                                                                                                                                                                                   
}                                                  
```


# Historical Performance Benchmarks

This document tracks the evolution of Aegisr's performance across versions. Metrics are collected using standard benchmarking tests for `put_value` and `get_value` operations.

## Benchmark Table

| **Version** | **Date** | **Put Ops/sec** | **Get Ops/sec** | **Put Latency (µs)** | **Get Latency (µs)** | **Notes** |
|-------------|----------|----------------|----------------|--------------------|--------------------|-----------|
| 1.0.1-beta  | 2025-11-17 | ~7,000          | ~100,000        | 136–145               | 9–10                  | Initial benchmark with 100 measurements. Found some outliers (7% put, 8% get). |
| 1.0.2-beta  | 2025-11-18 | ~122,000        | ~127,000        | 4.56–5.06             | 4.11–4.17             | Huge performance jump due to removal of autosave from every put/get. Outliers: 7–8% mild/high. |


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

## About Developer / Aegisr

Aegisr KVD is developed and maintained by Harold Eustaquio, an individual dedicated to building high-performance, reliable, and easy-to-use data storage solutions. For more information, visit his [website](https://harold-eustaquio.vercel.app/).

The name "Aegisr" is inspired by "Aegis," symbolizing protection and support, reflecting the database's goal of providing a robust and reliable data storage solution. 

For questions, support, or contributions, please reach out via [GitHub](https://github.com/surelle-ha/).
