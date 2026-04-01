# Snapshots

This directory holds the checked-in expected output for arcproof.

Generate or refresh it with:

```bash
cargo run --release -- refresh
```

Then verify it with:

```bash
cargo run --release -- check
```

Text snapshots live in `snapshots/text/`.
JSON snapshots live in `snapshots/json/`.
