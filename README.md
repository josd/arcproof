# ARC

ARC is a collection of small, self-contained C programs.
Each program answers one precise question, explains the result briefly, and
then checks that result with explicit validations.

That is the core ARC idea:

- **Answer** — compute the result
- **Reason** — show a short witness, derivation, or explanation
- **Check** — run explicit tests that can fail loudly if the result is wrong

The point is not only to compute something, but to compute it in a way that is
**easy to inspect, rerun, and verify**.

## What this project contains

The `examples/` directory contains 16 standalone C programs:

- `collatz_1000.c`
- `control_system.c`
- `deep_taxonomy_100000.c`
- `delfour.c`
- `euler_identity.c`
- `fibonacci.c`
- `goldbach_1000.c`
- `gps.c`
- `kaprekar_6174.c`
- `matrix_mechanics.c`
- `odrl_dpv_ehds_risk_ranked.c`
- `path_discovery.c`
- `pn_junction_tunneling.c`
- `polynomial.c`
- `transistor_switch.c`
- `sudoku.c`

Each file is meant to stand on its own:

- it has its own `main()`
- it keeps its own data and logic
- it can be compiled and run directly

## Why ARC is useful

This style keeps the important parts visible:

- the **data** is explicit
- the **logic** is local to the example
- the **question** is precise
- the **checks** are part of the running artifact

That makes the programs easier to audit, benchmark, teach from, and compare.
A good ARC check is not decorative: it should be a concrete test that can fail.

## Build

Build all examples:

```sh
make
```

This creates executables in `bin/`.

Build one example manually:

```sh
cc -O2 -std=c11 -Wall -Wextra -pedantic examples/sudoku.c -o bin/sudoku -lm -lcrypto -lgmp
```

## Run

Run any example directly:

```sh
./bin/sudoku
./bin/gps
./bin/path_discovery
```

Each program prints a human-readable ARC report.

## Check and timing

Run the project check:

```sh
make check
```

`make check` launches every built example once and prints a compact line for each one:

- example name
- one wall-clock timing value in milliseconds
- `OK`, `FAIL`, or `MISSING`

### Meaning of the check output

- `OK` — the example exited with code `0`
- `FAIL` — the example ran but returned a non-zero exit code
- `MISSING` — the executable was not built

The reported time is a single wall-clock run of the whole standalone executable,
in milliseconds. It includes process startup and normal scheduling effects, so
it is best read as a practical runtime number rather than a microbenchmark of
only the inner algorithm.

The timing helper is:

```sh
bash tools/check_with_timing.sh
```

To let each program print its normal output during timing:

```sh
bash tools/check_with_timing.sh --verbose
```

## Dependencies

Most examples only need the C standard library and `libm`.

Two examples currently use extra libraries:

- `fibonacci.c` uses **GMP** for exact big integer arithmetic
- `delfour.c` uses **OpenSSL libcrypto** for SHA-256 and HMAC-SHA256

Typical Debian/Ubuntu packages:

```sh
sudo apt-get install build-essential libgmp-dev libssl-dev
```

## Repository layout

```text
examples/   standalone C example programs
bin/        compiled executables produced by `make`
tools/      helper scripts, including the timing checker
Makefile    build and check entry point
README.md   project overview
```

## Included examples

The example set covers several kinds of problems:

- number theory: Collatz, Goldbach, Kaprekar, Fibonacci
- symbolic or algebraic reasoning: Euler identity, polynomial consistency
- search and planning: GPS, path discovery, Sudoku
- rule-based or policy-style reasoning: control system, EHDS risk ranking, Delfour
- small scientific toy models: matrix mechanics, PN junction tunneling, transistor switch
- scaling benchmark: deep taxonomy

## Project goal

The goal is simple:

- write small programs that answer a precise question
- make the reasoning visible in the program output
- validate the result with explicit checks
- keep each example easy to inspect and rerun
