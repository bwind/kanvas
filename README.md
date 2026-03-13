# kanvas

Declarative data visualization for Rust, compiled to Vega-Lite.

`kanvas` is a small Rust charting crate with a builder-style API that compiles charts to Vega-Lite.

## Example

```rust
use kanvas::*;
use serde_json::json;

let rows = vec![
    json!({"bucket": "A", "value": 12}),
    json!({"bucket": "B", "value": 19}),
    json!({"bucket": "C", "value": 7}),
];

let chart = Chart::new()
    .data(rows)
    .mark(Mark::Bar)
    .encode(
        Encoding::new()
            .x("bucket".n())
            .y("value".q())
            .color("bucket".n()),
    )
    .width(600)
    .height(400)
    .title("Bogus Values");

chart.show_in_browser("target/kanvas/chart.html")?;
```

## Preview

```bash
cargo run --example preview
```
