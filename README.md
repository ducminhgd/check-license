# Check License

A CLI tool that audits installed applications on your machine, identifies their license type, and flags suspected crack software or apps not permitted for commercial use.

## How it works

1. **Scans** all installed applications for the current OS (macOS `.app` bundles, Linux `.desktop` files, Windows registry entries).
2. **Resolves** the license model using two sources in priority order:
   - **Knowledge base** (`data/known_apps.json`) — curated records for popular apps including crack indicators.
   - **System package manager** — queries Homebrew (macOS), RPM/dpkg (Linux) for SPDX license identifiers and classifies OSS apps automatically.
3. **Checks** crack indicators: blocked activation domains in `/etc/hosts` and known patcher apps.
4. **Reports** a table and summary, exits with a non-zero code if issues are found.

## Output

A table with columns:

| Column | Description |
|--------|-------------|
| Application | Name of the application |
| Version | Installed version |
| License | License model: Free, Freemium, Paid, Open Source, Unknown |
| Activation | Activation status: App Store, Licensed, Unactivated, N/A, Unknown |
| Work Use | *(only with `--work`)* Whether the app is permitted for commercial use |
| Notes | Additional information |

## OS Support

- [x] macOS
- [x] Linux
- [x] Windows

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70 or later (`rustup` is the recommended installer)

## Usage

### Run from source

```bash
cargo run
```

With flags:

```bash
cargo run -- --work          # also flag apps not permitted for commercial use
cargo run -- --online        # allow network requests for unresolved apps (future)
cargo run -- --work --online
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
./target/release/check-license --online
```

Optionally, copy it to a directory on your `PATH` so it is available system-wide:

```bash
cp target/release/check-license /usr/local/bin/check-license
check-license
check-license --work
```

## Options

| Flag | Description |
|------|-------------|
| `--work` | Also flag applications whose license does not permit commercial use |
| `--online` | Allow network requests to fetch license data for apps not resolved offline |

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | All clear |
| `1` | One or more suspected crack applications detected |
| `2` | One or more applications not permitted for commercial use (only with `--work`) |

## License resolution logic

```
For each installed app:
  1. Look up in knowledge base by bundle ID, then by name
     → use KB record (license model, work_allowed, crack indicators)
  2. If not in KB, query system package manager for SPDX identifier
     → if SPDX is an OSS license (MIT, Apache, GPL, BSD, …) → OpenSource, work_allowed = true
  3. Otherwise → Unknown
```

## Knowledge base

`data/known_apps.json` contains curated records for popular applications. Each entry includes:

- `bundle_id` — macOS/Linux bundle identifier (null for Windows entries matched by name)
- `name` — display name
- `license_model` — `Free`, `Freemium`, `Paid`, `OpenSource`, or `Unknown`
- `work_allowed` — whether the app is permitted for commercial/work use
- `notes` — additional context
- `crack_indicators` *(optional)* — `hosts_entries` (activation domains) and `known_crack_app_bundle_ids` (known patchers)

To add an entry, edit `data/known_apps.json` and rebuild.
