# Check license

A tool to check all the applications of current machine, and list out all of their licenses. Raise error if there is any crack applications, or applications violate the license.

## Output

A table with columns:
- Appplication: name of the application
- Version: verson of allication.
- License/Plan: current license or plan of the application
- If there is a flag `--work` then allow it to check if the application can be used for work environment.
- Note: more information.

## OS Supported

- [x] MacOS
- [ ] Windows
- [ ] Linux

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70 or later (`rustup` is the recommended installer)

## Usage

### Run from source

```bash
cargo run
```

With the `--work` flag to also flag apps not permitted for commercial use:

```bash
cargo run -- --work
```

### Build a release binary

```bash
cargo build --release
```

The binary is placed at `target/release/check-license`.

### Run the binary

```bash
./target/release/check-license
./target/release/check-license --work
```

Optionally, copy it to a directory on your `PATH` so it is available system-wide:

```bash
cp target/release/check-license /usr/local/bin/check-license
check-license
check-license --work
```

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | All clear |
| `1` | One or more suspected crack applications detected |
| `2` | One or more applications not permitted for commercial use (only with `--work`) |
