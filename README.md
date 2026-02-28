# LC-3 Virtual Machine (Rust)

A Rust implementation of a virtual machine for the **LC-3 (Little Computer 3)** architecture. This project was made following the tutorial https://www.jmeiners.com/lc3-vm/.

It is capable of running the games 2048 and rogue LC-3.

---

## Build

```bash
cargo build --release
```

---

## Run

```bash
cargo run -- path/to/program.obj
```

Example:

```bash
cargo run -- 2048.obj
```

