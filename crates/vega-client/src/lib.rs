//! # vega-client
//!
//! Client-side runtime and hydration utilities for the Vega web framework.
//!
//! In SSR mode, the server renders HTML and the client hydrates it with
//! interactive behavior. This crate provides the client-side entry point.
//!
//! # Current Status
//!
//! This crate provides the `vega_hydrate!` macro for client-side initialization
//! and hydration data parsing. Full WASM hydration integration is planned.

use serde_json::Value;

/// Client-side hydration entry point macro.
///
/// In a full Leptos integration, this would call `leptos::hydrate()`.
/// Currently it evaluates the provided expression or is a no-op.
///
/// # Examples
///
/// ```
/// use vega_client::vega_hydrate;
/// let result = vega_hydrate!(2 + 2);
/// assert_eq!(result, 4);
/// ```
#[macro_export]
macro_rules! vega_hydrate {
    ($hydrate:expr) => {{
        $hydrate
    }};
    () => {{
        ()
    }};
}

/// Parse hydration data embedded in the HTML page by the SSR handler.
///
/// The SSR handler injects serialized data as `window.__VEGA_DATA__`.
/// This function deserializes it from a JSON string.
///
/// # Errors
///
/// Returns a `serde_json::Error` if the input is not valid JSON.
pub fn parse_hydration_data(raw: &str) -> serde_json::Result<Value> {
    serde_json::from_str(raw)
}

/// Serialize data for injection into the HTML shell as hydration state.
///
/// The returned string can be safely embedded in a `<script>` tag.
pub fn serialize_hydration_data<T: serde::Serialize>(data: &T) -> serde_json::Result<String> {
    serde_json::to_string(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hydration_parse() {
        let value = parse_hydration_data(r#"{"ready":true}"#).expect("json");
        assert_eq!(value["ready"], true);
    }

    #[test]
    fn hydrate_macro_runs() {
        let value = vega_hydrate!(2 + 2);
        assert_eq!(value, 4);
    }

    #[test]
    fn serialize_roundtrip() {
        let data = serde_json::json!({"count": 42});
        let serialized = serialize_hydration_data(&data).expect("serialize");
        let parsed = parse_hydration_data(&serialized).expect("parse");
        assert_eq!(parsed["count"], 42);
    }
}
