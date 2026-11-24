//! Settings for Sucrose.
//!
//! `legacy` is the read-only reader of the C# app's `Setting/*.json` files (FMT-08). `model` is the new versioned
//! settings format, `migrate` builds it once from the legacy files and `store` keeps `Setting/Sucrose.json` with
//! atomic writes (D-01, D-02, D-03).

pub mod legacy;
pub mod migrate;
pub mod model;
pub mod store;
