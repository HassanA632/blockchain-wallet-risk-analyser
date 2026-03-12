# blockchain-wallet-risk-analyser

A Rust CLI tool for analysing wallet exposure to risky blockchain entities.

Given a target wallet, the tool looks for direct and indirect connections within a chosen hop depth, checks discovered wallets against built-in and analyst supplied risk lists and returns a JSON report with findings and summary counts.

The current version uses local graph data to model wallet interactions while the core analysis engine is being developed. The project is structured so the local graph input can later be replaced with real Ethereum transaction data.

## Features

- analyse a target wallet at 1-hop or 2-hop depth
- detect exposure to known risky wallets
- support both built-in and analyst supplied risk lists
- distinguish between risk category and intelligence source
- aggregate repeated transfers into wallet-level relationships
- preserve wallet paths for risky findings
- filter graph data by optional date range
- output structured JSON to stdout or a file

## Current risk model

Risk categories currently used:

- `Sanctioned`
- `Mixer`
- `Suspect`
- `Other`

Each intelligence record also has a source:

- `BuiltIn`
- `Custom`

The severity model is intentionally simple:

- direct sanctioned exposure -> `High`
- direct mixer / suspect / other exposure -> `Medium`
- 2-hop risky exposure -> `Low`

## How it works

The current pipeline is:

1. load transaction edge data
2. optionally filter edges by date range
3. aggregate raw transfers into wallet relationships
4. traverse relationships from the target wallet
5. match discovered wallets against the risk index
6. build a JSON report

The analysis is wallet based rather than transaction based. Multiple transfers between the same wallet pair are grouped into a single relationship for traversal while transaction counts, assets seen, timestamps, and directional totals are kept as supporting evidence.

## Requirements

- Rust
- Cargo

## Run locally

```bash
cargo run -- \
  --chain ethereum \
  --wallet 0x1111111111111111111111111111111111111111 \
  --hops 2 \
  --custom-risk-list data/custom_risk_entities.json
```

Write output to a file:

```bash
cargo run -- \
  --chain ethereum \
  --wallet 0x1111111111111111111111111111111111111111 \
  --hops 2 \
  --custom-risk-list data/custom_risk_entities.json \
  --output output/report.json
```

## CLI arguments

| Argument             | Required | Description                                                          |
| -------------------- | -------- | -------------------------------------------------------------------- |
| `--chain`            | yes      | blockchain context for the analysis (`ethereum` currently supported) |
| `--wallet`           | yes      | target wallet address                                                |
| `--hops`             | yes      | hop depth to analyse (`1` or `2`)                                    |
| `--graph`            | no       | path to a graph JSON file; defaults to `data/sample_graph.json`      |
| `--custom-risk-list` | no       | path to an analyst-defined risk list                                 |
| `--output`           | no       | write JSON output to a file instead of stdout                        |
| `--from-date`        | no       | lower timestamp bound in `YYYY-MM-DDTHH:MM:SSZ` format               |
| `--to-date`          | no       | upper timestamp bound in `YYYY-MM-DDTHH:MM:SSZ` format               |

## Sample data

The project currently ships with small local datasets under `data/`:

- `sample_graph.json`
- `risk_entities.json`
- `custom_risk_entities.json`

These are mainly there to make the analysis engine easier to test and iterate on before I integrate a live Ethereum data source.

## Testing

Run the test suite with:

```bash
cargo test
```

The project currently includes unit tests for traversal, risk matching, reporting, relationship aggregation, filtering, validation, and loaders.

## Design notes

I made a few deliberate choices in the current version:

- wallet exposure is the main unit of analysis
- traversal runs on aggregated wallet relationships, not raw transfers
- built-in intelligence and analyst intelligence are separated by source
- addresses are normalised internally for consistent matching
- date filtering happens before relationship aggregation

## Limitations

Current limitations:

- local JSON graph input is used instead of a live Ethereum provider
- the scoring model is intentionally simple
- values are aggregated using a lightweight numeric approach suitable for the current local-data phase
- Ethereum address validation is format based rather than checksum based

## Next steps

Planned improvements include:

- integrating a real Ethereum data source
- replacing local sample graph input with live transfer data
- improving scoring and analyst context
- adding integration tests around full CLI runs
- UI integration

## Project goal

The aim of the project is to build a practical analyst-facing CLI tool rather than just a graph traversal demo.

The focus is on quick explainable exposure analysis:

- who the target wallet is connected to
- how close risky wallets are
- what type of risk was found
- where the intelligence came from
- what relationship evidence supports the connection
