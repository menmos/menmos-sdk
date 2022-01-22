# menmos-sdk

High-level rust client for writing applications using the menmos platform

## Compatibility Guarantees

Until menmos reaches 1.0, the only version of menmos that is _guaranteed_ to
work with this crate is the version of the menmos client that is pinned in this
package's Cargo.toml. Other versions _might_ work, but we won't test them.

Once menmos reaches 1.0, version 1.x of this crate will work for all menmos
versions `<= 2.0`.
