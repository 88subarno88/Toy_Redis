# Toy Redis (Rust)

A fully functional, concurrent, and persistent in-memory data structure store built entirely from scratch in Rust. It speaks the official RESP (REdis Serialization Protocol), meaning it is fully compatible with standard Redis clients like `redis-cli`.

This project was built to explore low-level systems programming, network socket handling, concurrent data structures, and message brokering in Rust.

---

## Core Features

- **RESP Compatible:** Communicates flawlessly with official enterprise tools (e.g., `redis-cli`).
- **Concurrent & Thread-Safe:** Handles multiple client connections simultaneously using heavily optimized `Arc` and `RwLock` primitives.
- **Database Sharding:** The internal storage engine is divided into 16 isolated HashMaps to eliminate thread-locking bottlenecks and maximize throughput.
- **Garbage Collection (TTL):** Features a background thread that actively monitors and aggressively sweeps expired keys from memory.
- **Disk Persistence (AOF):** Implements an Append-Only File engine using a background multi-producer, single-consumer (`mpsc`) channel. It survives aggressive kernel-level crashes and seamlessly rebuilds the memory state on boot.
- **Message Broker (Pub/Sub):** Full support for real-time `PUBLISH` and `SUBSCRIBE` broadcasting using asynchronous channel routing.

---

## Prerequisites

To run this server, you will need:

- **Rust & Cargo:** [Install Rust](https://www.rust-lang.org/tools/install)
- **Redis CLI:** Used to interact with the database.
  - _Ubuntu/Debian:_ `sudo apt install redis-tools`
  - _Mac (Homebrew):_ `brew install redis`

---

## Run Instructions

### 1. Start the Server

Clone the repository, navigate to the project directory, and start the database engine:

```bash
cargo run
```

The server will boot up, restore any saved data from `appendonly.aof`, and listen on `127.0.0.1:6379`.

### 2. Connect to the Database

Open a new terminal window and connect using the official Redis client:

```bash
redis-cli
```

---

## Supported Commands

This engine supports the most critical operations of a standard Redis instance.

### Key-Value Operations

```
> SET master_key "persistence"
OK
> GET master_key
"persistence"
> EXISTS master_key
(integer) 1
> DEL master_key
(integer) 1
> KEYS *
1) "master_key"
```

### Expiry & Garbage Collection

```
> SET bomb "tick tock"
OK
> EXPIRE bomb 30
(integer) 1
> TTL bomb
(integer) 29
```

### Real-Time Pub/Sub (Message Broker)

> To test this, open **two separate terminal windows**!

**Terminal A — The Subscriber:**

```
> SUBSCRIBE news
Reading messages... (press Ctrl-C to quit)
1) "subscribe"
2) "news"
3) (integer) 1
```

**Terminal B — The Publisher:**

```
> PUBLISH news "Rust is awesome!"
(integer) 1
```

> The message will instantly appear in Terminal A!

---

## Project Architecture

| File / Directory | Responsibility                                           |
| ---------------- | -------------------------------------------------------- |
| `server.rs`      | TCP connection handler and multi-threading router        |
| `commands/`      | RESP parser and command execution logic                  |
| `store/`         | Sharded `Arc<RwLock<HashMap>>` memory engine             |
| `expiry.rs`      | Background garbage collection daemon                     |
| `aof.rs`         | Append-Only File persistence engine and boot-up restorer |
| `pubsub.rs`      | Global `mpsc` message broker and channel registry        |

---

> Built with Rust. No external database libraries were used.

## Benchmarks

View the full [Criterion.rs Benchmark Report](https://88subarno88.github.io/toy_redis/target/criterion/report/index.html).
