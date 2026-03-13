use kanvas::*;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rows = vec![
        json!({"bucket": "A", "value": 12}),
        json!({"bucket": "B", "value": 19}),
        json!({"bucket": "C", "value": 7}),
        json!({"bucket": "D", "value": 23}),
        json!({"bucket": "E", "value": 15}),
        json!({"bucket": "F", "value": 9}),
        json!({"bucket": "G", "value": 18}),
    ];

    let chart = Chart::new()
        .data(rows)
        .config(
            ChartConfig::new()
                .bar_discrete_band_size(40)
                .bar_corner_radius_end(10)
                .band_padding_outer(0.4),
        )
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

    chart.show_in_browser("target/kanvas/bogus-bars.html")?;

    Ok(())
}
