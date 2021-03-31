# MPC: secret ranking of 3 players along 5 criteria


This programs shows how to secretly calculate for each criteria:
- the sum and reveal it to all players
- the rank of each player and reveal it to that player only

## Input data

Each player should provide input data with rows of
of the given criteria values rounded to the nearest Euro and where
the first column is expected to represent the month in `YYYYMM` format
e.g. `202102` for February 2021.

The lines must be sorted by month in ascending order

The output table for every player looks like

    Month, Value Criteria1, Sum criteria 1, Rank Criteria 1,...,Value Criteria5, Sum criteria 5, Rank Criteria


## Building

This example requires rust nightly > 2021-02-25

The WASM target must be installed:

```
rustup target add wasm32-unknown-unknown
```

Run `cargo vendor` once


After editing the code, use `build.sh` to compile