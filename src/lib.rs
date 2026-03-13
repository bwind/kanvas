#![forbid(unsafe_code)]
#![doc = "Declarative data visualization for Rust, compiled to Vega-Lite."]

use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use serde::Serialize;
use serde_json::{Map, Value, json};

/// A lightweight chart builder that compiles into a Vega-Lite specification.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Chart {
    title: Option<String>,
    mark: Option<Mark>,
    encoding: Encoding,
    data: Option<Value>,
    config: ChartConfig,
    width: Option<u32>,
    height: Option<u32>,
}

impl Chart {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn data<T>(self, data: T) -> Self
    where
        T: Serialize,
    {
        self.try_data(data)
            .expect("chart data must be serializable to JSON")
    }

    pub fn try_data<T>(mut self, data: T) -> Result<Self, serde_json::Error>
    where
        T: Serialize,
    {
        self.data = Some(serde_json::to_value(data)?);
        Ok(self)
    }

    #[must_use]
    pub fn mark(mut self, mark: Mark) -> Self {
        self.mark = Some(mark);
        self
    }

    #[must_use]
    pub fn encode(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
        self
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    pub fn config(mut self, config: ChartConfig) -> Self {
        self.config = config;
        self
    }

    #[must_use]
    pub fn text_color(mut self, color: impl Into<String>) -> Self {
        self.config.text_color = color.into();
        self
    }

    #[must_use]
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    #[must_use]
    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    #[must_use]
    pub fn compile(&self) -> VegaLiteSpec {
        VegaLiteSpec {
            title: self.title.clone(),
            mark: self.mark,
            encoding: self.encoding.clone(),
            data: self.data.clone(),
            config: self.config.clone(),
            width: self.width,
            height: self.height,
        }
    }

    #[must_use]
    pub fn to_vega_lite_json(&self) -> String {
        self.compile().to_json()
    }

    #[must_use]
    pub fn to_html(&self) -> String {
        self.compile().to_html()
    }

    pub fn write_html<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        self.compile().write_html(path)
    }

    pub fn show_in_browser<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        self.compile().show_in_browser(path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Line,
    Bar,
    Point,
    Area,
}

impl Mark {
    #[must_use]
    pub fn as_vega_lite(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Bar => "bar",
            Self::Point => "point",
            Self::Area => "area",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartConfig {
    text_color: String,
    categorical_palette: Vec<String>,
    bar_corner_radius_end: u32,
    bar_discrete_band_size: Option<u32>,
    band_padding_outer: Option<f64>,
}

impl ChartConfig {
    pub const DEFAULT_TEXT_COLOR: &'static str = "#333333";
    pub const DEFAULT_CATEGORICAL_PALETTE: [&'static str; 10] = [
        "#4e79a7", "#f28e2b", "#e15759", "#76b7b2", "#59a14f", "#edc948", "#b07aa1", "#ff9da7",
        "#9c755f", "#bab0ab",
    ];
    pub const DEFAULT_BAR_CORNER_RADIUS_END: u32 = 6;

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn text_color(mut self, color: impl Into<String>) -> Self {
        self.text_color = color.into();
        self
    }

    #[must_use]
    pub fn categorical_palette<I, S>(mut self, colors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.categorical_palette = colors.into_iter().map(Into::into).collect();
        self
    }

    #[must_use]
    pub fn bar_corner_radius_end(mut self, radius: u32) -> Self {
        self.bar_corner_radius_end = radius;
        self
    }

    #[must_use]
    pub fn bar_discrete_band_size(mut self, size: u32) -> Self {
        self.bar_discrete_band_size = Some(size);
        self
    }

    #[must_use]
    pub fn band_padding_outer(mut self, padding: f64) -> Self {
        self.band_padding_outer = Some(padding);
        self
    }

    fn to_value(&self) -> Map<String, Value> {
        let text_color = self.text_color.as_str();
        let mut value = json!({
            "title": {
                "color": text_color,
            },
            "axis": {
                "labelColor": text_color,
                "titleColor": text_color,
            },
            "legend": {
                "labelColor": text_color,
                "titleColor": text_color,
            },
            "header": {
                "labelColor": text_color,
                "titleColor": text_color,
            },
            "style": {
                "guide-label": {
                    "fill": text_color,
                },
                "guide-title": {
                    "fill": text_color,
                },
                "group-title": {
                    "fill": text_color,
                },
            },
            "text": {
                "fill": text_color,
            },
            "range": {
                "category": self.categorical_palette,
            },
            "bar": {
                "cornerRadiusEnd": self.bar_corner_radius_end,
            },
            "scale": {},
        });

        match &mut value {
            Value::Object(root) => {
                if let Some(size) = self.bar_discrete_band_size {
                    let bar = root
                        .get_mut("bar")
                        .and_then(Value::as_object_mut)
                        .expect("bar config should be a JSON object");
                    bar.insert("discreteBandSize".to_string(), json!(size));
                }

                if let Some(padding) = self.band_padding_outer {
                    let scale = root
                        .get_mut("scale")
                        .and_then(Value::as_object_mut)
                        .expect("scale config should be a JSON object");
                    scale.insert("bandPaddingOuter".to_string(), json!(padding));
                }
            }
            _ => unreachable!("chart config should serialize to a JSON object"),
        }

        match value {
            Value::Object(map) => map,
            _ => unreachable!("chart config should serialize to a JSON object"),
        }
    }
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            text_color: Self::DEFAULT_TEXT_COLOR.to_string(),
            categorical_palette: Self::DEFAULT_CATEGORICAL_PALETTE
                .into_iter()
                .map(str::to_string)
                .collect(),
            bar_corner_radius_end: Self::DEFAULT_BAR_CORNER_RADIUS_END,
            bar_discrete_band_size: None,
            band_padding_outer: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Encoding {
    x: Option<Field>,
    y: Option<Field>,
    color: Option<Field>,
}

impl Encoding {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn x(mut self, field: Field) -> Self {
        self.x = Some(field);
        self
    }

    #[must_use]
    pub fn y(mut self, field: Field) -> Self {
        self.y = Some(field);
        self
    }

    #[must_use]
    pub fn color(mut self, field: Field) -> Self {
        self.color = Some(field);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    name: String,
    kind: FieldType,
}

impl Field {
    #[must_use]
    pub fn new(name: impl Into<String>, kind: FieldType) -> Self {
        Self {
            name: name.into(),
            kind,
        }
    }

    #[must_use]
    pub fn temporal(name: impl Into<String>) -> Self {
        Self::new(name, FieldType::Temporal)
    }

    #[must_use]
    pub fn quant(name: impl Into<String>) -> Self {
        Self::new(name, FieldType::Quantitative)
    }

    #[must_use]
    pub fn nominal(name: impl Into<String>) -> Self {
        Self::new(name, FieldType::Nominal)
    }

    #[must_use]
    pub fn ordinal(name: impl Into<String>) -> Self {
        Self::new(name, FieldType::Ordinal)
    }
}

/// Shorthand methods for constructing typed fields from string names.
pub trait FieldShorthand {
    #[must_use]
    fn t(&self) -> Field;

    #[must_use]
    fn q(&self) -> Field;

    #[must_use]
    fn n(&self) -> Field;

    #[must_use]
    fn o(&self) -> Field;
}

impl FieldShorthand for str {
    fn t(&self) -> Field {
        Field::temporal(self)
    }

    fn q(&self) -> Field {
        Field::quant(self)
    }

    fn n(&self) -> Field {
        Field::nominal(self)
    }

    fn o(&self) -> Field {
        Field::ordinal(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Temporal,
    Quantitative,
    Nominal,
    Ordinal,
}

impl FieldType {
    #[must_use]
    pub fn as_vega_lite(self) -> &'static str {
        match self {
            Self::Temporal => "temporal",
            Self::Quantitative => "quantitative",
            Self::Nominal => "nominal",
            Self::Ordinal => "ordinal",
        }
    }
}

/// Placeholder compiled Vega-Lite representation.
#[derive(Debug, Clone, PartialEq)]
pub struct VegaLiteSpec {
    title: Option<String>,
    mark: Option<Mark>,
    encoding: Encoding,
    data: Option<Value>,
    config: ChartConfig,
    width: Option<u32>,
    height: Option<u32>,
}

impl VegaLiteSpec {
    pub const SCHEMA_URL: &'static str = "https://vega.github.io/schema/vega-lite/v6.json";
    pub const VEGA_CDN_URL: &'static str = "https://cdn.jsdelivr.net/npm/vega@6";
    pub const VEGA_LITE_CDN_URL: &'static str = "https://cdn.jsdelivr.net/npm/vega-lite@6";
    pub const VEGA_EMBED_CDN_URL: &'static str = "https://cdn.jsdelivr.net/npm/vega-embed@7";

    #[must_use]
    pub fn to_json(&self) -> String {
        self.to_value().to_string()
    }

    #[must_use]
    pub fn to_html(&self) -> String {
        let title = self.title.as_deref().unwrap_or("kanvas preview");
        let spec_json = escape_json_for_html_script(&self.to_json());

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{}</title>
    <script src="{}"></script>
    <script src="{}"></script>
    <script src="{}"></script>
    <style>
      :root {{
        color-scheme: light;
      }}

      body {{
        margin: 0;
        padding: 16px;
        background: #ffffff;
        color: #000000;
        font-family: sans-serif;
      }}

      #vis {{
        max-width: 960px;
        margin: 0 auto;
      }}
    </style>
  </head>
  <body>
    <div id="vis"></div>
    <script>
      const spec = {};

      vegaEmbed('#vis', spec, {{ mode: 'vega-lite', actions: false }})
        .catch((error) => {{
          const pre = document.createElement('pre');
          pre.textContent = error.stack || String(error);
          document.body.appendChild(pre);
        }});
    </script>
  </body>
</html>
"#,
            escape_html(title),
            Self::VEGA_CDN_URL,
            Self::VEGA_LITE_CDN_URL,
            Self::VEGA_EMBED_CDN_URL,
            spec_json
        )
    }

    pub fn write_html<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }

        fs::write(path, self.to_html())
    }

    pub fn show_in_browser<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        self.write_html(path)?;
        open_in_browser(&path.canonicalize()?)
    }

    fn to_value(&self) -> Value {
        let mut spec = Map::new();
        spec.insert(
            "$schema".to_string(),
            Value::String(Self::SCHEMA_URL.to_string()),
        );

        if let Some(data) = &self.data {
            spec.insert("data".to_string(), json!({ "values": data }));
        }

        spec.insert("config".to_string(), Value::Object(self.config.to_value()));

        if let Some(width) = self.width {
            spec.insert("width".to_string(), json!(width));
        }

        if let Some(height) = self.height {
            spec.insert("height".to_string(), json!(height));
        }

        if let Some(title) = &self.title {
            spec.insert("title".to_string(), Value::String(title.clone()));
        }

        if let Some(mark) = self.mark {
            spec.insert(
                "mark".to_string(),
                Value::String(mark.as_vega_lite().to_string()),
            );
        }

        let encoding = self.encoding.to_value();
        if !encoding.is_empty() {
            spec.insert("encoding".to_string(), Value::Object(encoding));
        }

        Value::Object(spec)
    }
}

impl Encoding {
    fn to_value(&self) -> Map<String, Value> {
        let mut channels = Map::new();

        if let Some(field) = &self.x {
            channels.insert("x".to_string(), field.to_value());
        }

        if let Some(field) = &self.y {
            channels.insert("y".to_string(), field.to_value());
        }

        if let Some(field) = &self.color {
            channels.insert("color".to_string(), field.to_value());
        }

        channels
    }
}

impl Field {
    fn to_value(&self) -> Value {
        json!({
            "field": self.name,
            "type": self.kind.as_vega_lite(),
        })
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_json_for_html_script(value: &str) -> String {
    value
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

#[cfg(target_os = "macos")]
fn open_in_browser(path: &Path) -> io::Result<()> {
    let mut command = Command::new("open");
    command.arg(path);
    run_browser_command(command)
}

#[cfg(target_os = "linux")]
fn open_in_browser(path: &Path) -> io::Result<()> {
    let mut command = Command::new("xdg-open");
    command.arg(path);
    run_browser_command(command)
}

#[cfg(target_os = "windows")]
fn open_in_browser(path: &Path) -> io::Result<()> {
    let mut command = Command::new("cmd");
    command.args(["/C", "start", ""]).arg(path);
    run_browser_command(command)
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn open_in_browser(_path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "opening a browser is not supported on this platform",
    ))
}

fn run_browser_command(mut command: Command) -> io::Result<()> {
    let status = command.status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "browser command exited with status {status}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;
    use std::env;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn builder_api_matches_the_requested_shape() {
        let rows = vec![
            json!({"time": "2026-03-10T09:00:00Z", "spread": -0.8}),
            json!({"time": "2026-03-10T10:00:00Z", "spread": -0.2}),
            json!({"time": "2026-03-10T11:00:00Z", "spread": 0.6}),
        ];

        let chart = Chart::new()
            .data(rows)
            .mark(Mark::Line)
            .encode(Encoding::new().x("time".t()).y("spread".q()))
            .width(640)
            .height(320)
            .title("Spread");

        let spec: Value = from_str(&chart.to_vega_lite_json()).expect("chart JSON should parse");

        assert_eq!(spec["title"], "Spread");
        assert_eq!(spec["mark"], "line");
        assert_eq!(spec["encoding"]["x"]["field"], "time");
        assert_eq!(spec["encoding"]["y"]["field"], "spread");
        assert_eq!(spec["data"]["values"][0]["time"], "2026-03-10T09:00:00Z");
        assert_eq!(spec["width"], 640);
        assert_eq!(spec["height"], 320);
        assert_eq!(
            spec["config"]["axis"]["labelColor"],
            ChartConfig::DEFAULT_TEXT_COLOR
        );
        assert_eq!(
            spec["config"]["bar"]["cornerRadiusEnd"],
            ChartConfig::DEFAULT_BAR_CORNER_RADIUS_END
        );
    }

    #[test]
    fn shorthand_methods_preserve_field_types() {
        assert_eq!("time".t(), Field::temporal("time"));
        assert_eq!("spread".q(), Field::quant("spread"));
        assert_eq!("series".n(), Field::nominal("series"));
        assert_eq!("rank".o(), Field::ordinal("rank"));
    }

    #[test]
    fn html_output_bootstraps_vega_embed() {
        let chart = Chart::new()
            .data(vec![
                json!({"time": "2026-03-10T09:00:00Z", "spread": -0.8}),
            ])
            .mark(Mark::Line)
            .encode(Encoding::new().x("time".t()).y("spread".q()))
            .title("Spread");

        let html = chart.to_html();

        assert!(html.contains(VegaLiteSpec::VEGA_CDN_URL));
        assert!(html.contains(VegaLiteSpec::VEGA_LITE_CDN_URL));
        assert!(html.contains(VegaLiteSpec::VEGA_EMBED_CDN_URL));
        assert!(html.contains("vegaEmbed('#vis', spec"));
    }

    #[test]
    fn chart_uses_default_text_color_config() {
        let chart = Chart::new();
        let spec: Value = from_str(&chart.to_vega_lite_json()).expect("chart JSON should parse");

        assert_eq!(
            spec["config"]["title"]["color"],
            ChartConfig::DEFAULT_TEXT_COLOR
        );
        assert_eq!(
            spec["config"]["axis"]["titleColor"],
            ChartConfig::DEFAULT_TEXT_COLOR
        );
        assert_eq!(
            spec["config"]["legend"]["labelColor"],
            ChartConfig::DEFAULT_TEXT_COLOR
        );
        assert_eq!(
            spec["config"]["style"]["guide-label"]["fill"],
            ChartConfig::DEFAULT_TEXT_COLOR
        );
        assert_eq!(
            spec["config"]["text"]["fill"],
            ChartConfig::DEFAULT_TEXT_COLOR
        );
        assert_eq!(
            spec["config"]["range"]["category"][0],
            ChartConfig::DEFAULT_CATEGORICAL_PALETTE[0]
        );
        assert_eq!(
            spec["config"]["bar"]["cornerRadiusEnd"],
            ChartConfig::DEFAULT_BAR_CORNER_RADIUS_END
        );
    }

    #[test]
    fn chart_text_color_can_be_overridden() {
        let chart = Chart::new().text_color("#555555");
        let spec: Value = from_str(&chart.to_vega_lite_json()).expect("chart JSON should parse");

        assert_eq!(spec["config"]["title"]["color"], "#555555");
        assert_eq!(spec["config"]["axis"]["labelColor"], "#555555");
        assert_eq!(spec["config"]["style"]["group-title"]["fill"], "#555555");
    }

    #[test]
    fn categorical_palette_and_bar_radius_can_be_overridden() {
        let chart = Chart::new().config(
            ChartConfig::new()
                .categorical_palette(["#111111", "#222222", "#333333"])
                .bar_corner_radius_end(12),
        );
        let spec: Value = from_str(&chart.to_vega_lite_json()).expect("chart JSON should parse");

        assert_eq!(spec["config"]["range"]["category"][0], "#111111");
        assert_eq!(spec["config"]["range"]["category"][2], "#333333");
        assert_eq!(spec["config"]["bar"]["cornerRadiusEnd"], 12);
    }

    #[test]
    fn bar_band_size_and_outer_padding_can_be_overridden() {
        let chart = Chart::new().config(
            ChartConfig::new()
                .bar_discrete_band_size(48)
                .band_padding_outer(0.35),
        );
        let spec: Value = from_str(&chart.to_vega_lite_json()).expect("chart JSON should parse");

        assert_eq!(spec["config"]["bar"]["discreteBandSize"], 48);
        assert_eq!(spec["config"]["scale"]["bandPaddingOuter"], 0.35);
    }

    #[test]
    fn color_channel_is_emitted_when_present() {
        let chart = Chart::new().encode(
            Encoding::new()
                .x("bucket".n())
                .y("value".q())
                .color("bucket".n()),
        );
        let spec: Value = from_str(&chart.to_vega_lite_json()).expect("chart JSON should parse");

        assert_eq!(spec["encoding"]["color"]["field"], "bucket");
        assert_eq!(spec["encoding"]["color"]["type"], "nominal");
    }

    #[test]
    fn chart_dimensions_can_be_overridden() {
        let chart = Chart::new().width(800).height(480);
        let spec: Value = from_str(&chart.to_vega_lite_json()).expect("chart JSON should parse");

        assert_eq!(spec["width"], 800);
        assert_eq!(spec["height"], 480);
    }

    #[test]
    fn write_html_persists_a_preview_page() {
        let chart = Chart::new()
            .data(vec![
                json!({"time": "2026-03-10T09:00:00Z", "spread": -0.8}),
            ])
            .mark(Mark::Line)
            .encode(Encoding::new().x("time".t()).y("spread".q()))
            .title("Spread");

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after the unix epoch")
            .as_nanos();
        let path = env::temp_dir().join(format!("kanvas-{}-{unique}.html", process::id()));

        chart
            .write_html(&path)
            .expect("writing the preview page should succeed");

        let html = fs::read_to_string(&path).expect("preview page should be readable");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Spread"));

        let _ = fs::remove_file(path);
    }
}
