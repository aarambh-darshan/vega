use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParamsError {
    #[error("failed to decode params: {0}")]
    Decode(serde_urlencoded::de::Error),
    #[error("failed to decode search params: {0}")]
    SearchDecode(serde_urlencoded::de::Error),
    #[error("failed to encode search params: {0}")]
    SearchEncode(#[from] serde_urlencoded::ser::Error),
}

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("client mode requires HTTP bridge implementation")]
    ClientBridgeUnavailable,
    #[error(transparent)]
    Params(#[from] ParamsError),
}

#[derive(Debug, Clone)]
pub struct QueryState<T> {
    pub data: Option<T>,
    pub loading: bool,
    pub error: Option<String>,
}

impl<T> QueryState<T> {
    pub fn loading() -> Self {
        Self {
            data: None,
            loading: true,
            error: None,
        }
    }

    pub fn ready(data: T) -> Self {
        Self {
            data: Some(data),
            loading: false,
            error: None,
        }
    }

    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            data: None,
            loading: false,
            error: Some(message.into()),
        }
    }
}

pub struct Mutation<F> {
    handler: F,
}

impl<F> Mutation<F> {
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<F> Mutation<F> {
    pub async fn execute<Args, Fut, T, E>(&self, args: Args) -> Result<T, E>
    where
        F: Fn(Args) -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        (self.handler)(args).await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchParamMode {
    Replace,
    Push,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchParamResult {
    pub url: String,
    pub mode: SearchParamMode,
}

#[derive(Debug, Clone)]
pub struct SearchParamHandle {
    current_url: String,
    updates: HashMap<String, Option<String>>,
    clear_existing: bool,
    mode: SearchParamMode,
}

impl SearchParamHandle {
    pub fn new(current_url: impl Into<String>) -> Self {
        Self {
            current_url: current_url.into(),
            updates: HashMap::new(),
            clear_existing: false,
            mode: SearchParamMode::Replace,
        }
    }

    pub fn mode(mut self, mode: SearchParamMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn set(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.updates.insert(key.into(), Some(value.into()));
        self
    }

    pub fn remove(mut self, key: impl Into<String>) -> Self {
        self.updates.insert(key.into(), None);
        self
    }

    pub fn merge<I, K, V>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (key, value) in values {
            self.updates.insert(key.into(), Some(value.into()));
        }
        self
    }

    pub fn clear(mut self) -> Self {
        self.clear_existing = true;
        self.updates.clear();
        self
    }

    pub fn apply(self) -> SearchParamResult {
        let (path, mut query_map) = split_url_to_map(&self.current_url);
        if self.clear_existing {
            query_map.clear();
        }

        for (key, value) in self.updates {
            match value {
                Some(value) => {
                    query_map.insert(key, value);
                }
                None => {
                    query_map.remove(&key);
                }
            }
        }

        let query = encode_flat_query(&query_map);
        let url = if query.is_empty() {
            path
        } else {
            format!("{path}?{query}")
        };

        SearchParamResult {
            url,
            mode: self.mode,
        }
    }
}

pub fn search_params(current_url: impl Into<String>) -> SearchParamHandle {
    SearchParamHandle::new(current_url)
}

#[cfg(feature = "ssr")]
pub async fn fetch<F, Args, Fut, T>(server_fn: F, args: Args) -> Result<T, FetchError>
where
    F: FnOnce(Args) -> Fut,
    Fut: Future<Output = Result<T, FetchError>>,
{
    server_fn(args).await
}

#[cfg(not(feature = "ssr"))]
pub async fn fetch<F, Args, Fut, T>(_server_fn: F, _args: Args) -> Result<T, FetchError>
where
    F: FnOnce(Args) -> Fut,
    Fut: Future<Output = Result<T, FetchError>>,
{
    Err(FetchError::ClientBridgeUnavailable)
}

pub async fn use_query<F, Fut, T, E>(query: F) -> QueryState<T>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    match query().await {
        Ok(data) => QueryState::ready(data),
        Err(err) => QueryState::failed(err.to_string()),
    }
}

pub fn use_mutation<F>(handler: F) -> Mutation<F> {
    Mutation::new(handler)
}

pub fn use_params<T>(params: &HashMap<String, String>) -> Result<T, ParamsError>
where
    T: DeserializeOwned,
{
    decode_map(params).map_err(ParamsError::Decode)
}

pub fn use_search_params<T>(params: &HashMap<String, String>) -> Result<T, ParamsError>
where
    T: DeserializeOwned,
{
    decode_map(params).map_err(ParamsError::SearchDecode)
}

pub fn decode_search_query<T>(query: &str) -> Result<T, ParamsError>
where
    T: DeserializeOwned,
{
    serde_urlencoded::from_str(query).map_err(ParamsError::SearchDecode)
}

pub fn encode_search_query<T>(value: &T) -> Result<String, ParamsError>
where
    T: Serialize,
{
    serde_urlencoded::to_string(value).map_err(ParamsError::SearchEncode)
}

pub fn set_search_params(
    current_url: &str,
    updates: &HashMap<String, Option<String>>,
    mode: SearchParamMode,
) -> SearchParamResult {
    let mut handle = search_params(current_url.to_string()).mode(mode);
    for (key, value) in updates {
        handle = match value {
            Some(value) => handle.set(key.clone(), value.clone()),
            None => handle.remove(key.clone()),
        };
    }
    handle.apply()
}

pub fn remove_search_param(
    current_url: &str,
    key: &str,
    mode: SearchParamMode,
) -> SearchParamResult {
    search_params(current_url.to_string())
        .mode(mode)
        .remove(key.to_string())
        .apply()
}

pub fn clear_search_params(current_url: &str, mode: SearchParamMode) -> SearchParamResult {
    search_params(current_url.to_string())
        .mode(mode)
        .clear()
        .apply()
}

pub fn merge_search_params(
    current_url: &str,
    values: &HashMap<String, String>,
    mode: SearchParamMode,
) -> SearchParamResult {
    let pairs = values
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<Vec<_>>();

    search_params(current_url.to_string())
        .mode(mode)
        .merge(pairs)
        .apply()
}

fn decode_map<T>(params: &HashMap<String, String>) -> Result<T, serde_urlencoded::de::Error>
where
    T: DeserializeOwned,
{
    let encoded = encode_flat_query(params);
    serde_urlencoded::from_str(&encoded)
}

fn split_url_to_map(url: &str) -> (String, HashMap<String, String>) {
    let (path, query) = match url.split_once('?') {
        Some((path, query)) => (path.to_string(), query),
        None => (url.to_string(), ""),
    };

    let mut query_map = HashMap::new();
    if !query.is_empty() {
        for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
            query_map.insert(key.to_string(), value.to_string());
        }
    }

    (path, query_map)
}

fn encode_flat_query(map: &HashMap<String, String>) -> String {
    let mut pairs = map.iter().collect::<Vec<_>>();
    pairs.sort_by(|a, b| a.0.cmp(b.0));

    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in pairs {
        serializer.append_pair(key, value);
    }
    serializer.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Params {
        slug: String,
        id: u64,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct Search {
        page: usize,
        q: String,
    }

    #[tokio::test]
    async fn query_success() {
        let state = use_query(|| async { Ok::<_, FetchError>(42) }).await;
        assert_eq!(state.data, Some(42));
        assert!(!state.loading);
        assert!(state.error.is_none());
    }

    #[tokio::test]
    async fn mutation_runs() {
        let mutation = use_mutation(|value: i32| async move { Ok::<_, FetchError>(value + 1) });
        let result = mutation.execute(10).await.expect("result");
        assert_eq!(result, 11);
    }

    #[test]
    fn params_decode() {
        let mut map = HashMap::new();
        map.insert("slug".to_string(), "hello".to_string());
        map.insert("id".to_string(), "42".to_string());
        let parsed: Params = use_params(&map).expect("decode");
        assert_eq!(
            parsed,
            Params {
                slug: "hello".to_string(),
                id: 42
            }
        );
    }

    #[test]
    fn search_encode_decode() {
        let query = encode_search_query(&Search {
            page: 2,
            q: "rust".to_string(),
        })
        .expect("encode");
        let decoded: Search = decode_search_query(&query).expect("decode");
        assert_eq!(
            decoded,
            Search {
                page: 2,
                q: "rust".to_string()
            }
        );
    }

    #[test]
    fn set_search_params_updates() {
        let mut updates = HashMap::new();
        updates.insert("page".to_string(), Some("3".to_string()));
        updates.insert("q".to_string(), None);

        let result = set_search_params("/blog?page=1&q=rust", &updates, SearchParamMode::Replace);
        assert_eq!(result.url, "/blog?page=3");
        assert_eq!(result.mode, SearchParamMode::Replace);
    }

    #[test]
    fn clear_search_params_works() {
        let result = clear_search_params("/blog?page=2", SearchParamMode::Push);
        assert_eq!(result.url, "/blog");
        assert_eq!(result.mode, SearchParamMode::Push);
    }

    #[test]
    fn search_param_builder_remove_and_merge() {
        let result = search_params("/blog?page=1")
            .set("q", "rust")
            .merge([(String::from("sort"), String::from("new"))])
            .remove("page")
            .mode(SearchParamMode::Replace)
            .apply();

        assert_eq!(result.url, "/blog?q=rust&sort=new");
    }
}
