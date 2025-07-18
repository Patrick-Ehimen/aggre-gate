# Lib Module

## Overview

The lib module re-exports key components of the aggregator-core, expanding the public API surface.

## Re-exports

- Aggregator
- Config
- Error
- Types

These modules are re-exported to unify them under a single, accessible interface.

## Example: Using Re-exports

```rust
use aggregator_core::{Aggregator, Config};

let config = Config::default();
let aggregator = Aggregator::new(config);
```

## API Reference

See individual module documentation for further details on structures and functions provided.
Cross-link to:
- [Aggregator Documentation](aggregator.md)
- [Config Documentation](config.md)
- [Types Documentation](types.md)
- [Error Documentation](error.md)
