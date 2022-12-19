use std::collections::HashMap;

use anyhow::Context;
use anyhttp::{HttpError, Method, RequestBody};

// type Request = anyhttp::Request<RequestBody>;
type Response = anyhttp::Response<anyhttp::sync::DynResponseBody>;

use super::{
    types::{
        Direction, Timelog, TimelogFilter, TimelogOrder, TimelogQuery, User, UserFilter, UserQuery,
    },
    Db,
};

#[derive(Clone)]
pub struct SupaDb {
    endpoint: String,
    api_key: String,
    client: anyhttp::sync::DynClient,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct ApiError {
    pub message: String,
    pub code: String,
    pub details: Option<String>,
    pub hint: Option<String>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Supabase API error: {}", self.message)
    }
}

impl std::error::Error for ApiError {}

impl SupaDb {
    pub fn new(endpoint: String, api_key: String) -> Result<Self, anyhow::Error> {
        let client = crate::util::WasixHttpExecutor::new_dyn_client()?;
        let endpoint = endpoint
            .strip_suffix('/')
            .map(|s| s.to_string())
            .unwrap_or(endpoint);

        Ok(Self {
            endpoint,
            api_key,
            client,
        })
    }

    fn send(
        &self,
        mut pre: anyhttp::RequestPre<anyhttp::RequestBody>,
    ) -> Result<Response, HttpError> {
        let uri = pre.request.uri.to_string();
        let clean_path = uri.strip_prefix('/').map(|x| x.to_string()).unwrap_or(uri);
        pre.request.uri = format!("{}/{}", self.endpoint, clean_path).parse().unwrap();
        pre.request
            .headers
            .insert("apikey", self.api_key.parse().unwrap());
        eprintln!(
            "Sending http request: {} - {}",
            pre.request.uri, pre.request.method
        );
        self.client.send_pre(pre)
    }

    fn get_json<O>(&self, path: &str) -> Result<O, HttpError>
    where
        O: serde::de::DeserializeOwned,
    {
        let req = self
            .client
            .get(path)
            .header(http::header::CONTENT_TYPE, "application/json")
            .header(http::header::ACCEPT, "application/json")
            .body(RequestBody::Empty)
            .build()?;
        let body = self.send(req)?.error_for_status()?.bytes_sync()?;

        match serde_json::from_slice(&body) {
            Ok(o) => Ok(o),
            Err(err) => Err(HttpError::new_custom_with_cause(
                "could not deserialize response body",
                err,
            )),
        }
    }

    fn list_table<O>(&self, path: &str, limit: u64, offset: u64) -> Result<O, HttpError>
    where
        O: serde::de::DeserializeOwned,
    {
        let range = format!("{}-{}", offset, limit);

        let req = self
            .client
            .get(path)
            .header(http::header::CONTENT_TYPE, "application/json")
            .header(http::header::ACCEPT, "application/json")
            .header("Range", range)
            .body(RequestBody::Empty)
            .build()?;
        let body = self.send(req)?.error_for_status()?.bytes_sync()?;

        match serde_json::from_slice(&body) {
            Ok(o) => Ok(o),
            Err(err) => Err(HttpError::new_custom_with_cause(
                "could not deserialize response body",
                err,
            )),
        }
    }

    fn send_json_with_prefer_return<I, O>(
        &self,
        method: Method,
        path: &str,
        data: &I,
    ) -> Result<O, HttpError>
    where
        I: serde::Serialize,
        O: serde::de::DeserializeOwned,
    {
        let pre = self
            .client
            .request(method, path)
            .header(http::header::CONTENT_TYPE, "application/json")
            .header(http::header::ACCEPT, "application/json")
            .header("Prefer", "return=representation")
            .json(data)
            .build()?;
        let res = self.send(pre)?;
        let status = res.status;

        let body = res.bytes_sync()?;

        if !status.is_success() {
            if let Ok(err) = serde_json::from_slice::<ApiError>(&body) {
                return Err(HttpError::new_custom_with_cause("api request failed ", err));
            } else {
                return Err(HttpError::new_custom("api request failed "));
            }
        }

        match serde_json::from_slice(&body) {
            Ok(o) => Ok(o),
            Err(err) => {
                // let body_str = String::from_utf8_lossy(&body);

                Err(HttpError::new_custom_with_cause(
                    "could not deserialize response body",
                    err,
                ))
            }
        }
    }

    fn post_json_with_prefer_return<I, O>(&self, path: &str, data: &I) -> Result<O, HttpError>
    where
        I: serde::Serialize,
        O: serde::de::DeserializeOwned,
    {
        self.send_json_with_prefer_return(Method::POST, path, data)
    }

    fn patch_json_with_prefer_return<I, O>(&self, path: &str, data: &I) -> Result<O, HttpError>
    where
        I: serde::Serialize,
        O: serde::de::DeserializeOwned,
    {
        self.send_json_with_prefer_return(Method::PATCH, path, data)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(transparent)]
struct QueryMap(HashMap<String, Vec<String>>);

impl QueryMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.entry(key.into()).or_default().push(value.into());
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.insert(key.into(), vec![value.into()]);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0
            .iter()
            .map(|(key, vals)| vals.iter().map(|val| (key.as_str(), val.as_str())))
            .flatten()
    }

    pub fn to_query(&self) -> String {
        form_urlencoded::Serializer::new(String::new())
            .extend_pairs(self.iter())
            .finish()
    }
}

impl Default for QueryMap {
    fn default() -> Self {
        Self::new()
    }
}

fn build_user_filter(f: &UserFilter) -> QueryMap {
    let mut map = QueryMap::new();

    match f {
        UserFilter::Id(id) => {
            map.add("id", format!("eq.{id}"));
        }
        UserFilter::Name(name) => {
            map.add("username", format!("eq.{name}"));
        }
    }

    map
}

fn build_user_query(q: &UserQuery) -> QueryMap {
    let map = q.filter.as_ref().map(build_user_filter).unwrap_or_default();

    map
}

fn build_timelog_filter(f: &TimelogFilter) -> QueryMap {
    let mut map = QueryMap::new();
    build_timelog_filter_rec(f, &mut map);
    map
}

fn build_timelog_filter_rec(f: &TimelogFilter, map: &mut QueryMap) {
    match f {
        TimelogFilter::UserId(u) => {
            map.add("user_id", format!("eq.{u}"));
        }
        TimelogFilter::IsFinished(flag) => {
            if *flag {
                map.add("finished_at", "not.is.null");
            } else {
                map.add("finished_at", "is.null");
            }
        }
        TimelogFilter::And(items) => {
            for item in items {
                build_timelog_filter_rec(item, map);
            }
        }
        TimelogFilter::Id(id) => {
            map.add("id", format!("eq.{id}"));
        }
    }
}

fn build_timelog_query(q: &TimelogQuery) -> QueryMap {
    let mut map = q
        .filter
        .as_ref()
        .map(build_timelog_filter)
        .unwrap_or_default();

    for order in &q.order {
        let dir = match order.direction {
            Direction::Asc => "asc",
            Direction::Desc => "desc",
        };
        let col = match order.expr {
            TimelogOrder::Id => "id",
            TimelogOrder::StartedAt => "started_at",
        };
        map.add("order", format!("{col}.{dir}"))
    }

    map
}

impl Db for SupaDb {
    fn user(
        &self,
        filter: super::types::UserFilter,
    ) -> Result<Option<super::types::User>, anyhow::Error> {
        let mut qm = build_user_filter(&filter);
        qm.set("select", "*");

        let path = format!("/users?{}", qm.to_query());
        let users: Vec<User> = self.get_json(&path)?;
        let user = users.into_iter().next();
        Ok(user)
    }

    fn users(&self, query: super::types::UserQuery) -> Result<Vec<User>, anyhow::Error> {
        let mut qm = build_user_query(&query);
        qm.set("select", "*");

        let path = format!("/users?{}", qm.to_query());
        self.list_table(&path, query.limit, query.offset)
            .map_err(From::from)
    }

    fn user_create(
        &self,
        user: super::types::UserCreate,
    ) -> Result<super::types::User, anyhow::Error> {
        let users: Vec<User> = self.post_json_with_prefer_return("/users", &user)?;
        users
            .into_iter()
            .next()
            .context("API returned invalid data")
    }

    fn timelogs(
        &self,
        query: super::types::TimelogQuery,
    ) -> Result<Vec<super::types::Timelog>, anyhow::Error> {
        let mut qm = build_timelog_query(&query);
        qm.add("select", "*");
        let path = format!("/timelogs?{}", qm.to_query());

        self.list_table(&path, query.limit, query.offset)
            .map_err(From::from)
    }

    fn timelog_create(
        &self,
        log: super::types::TimelogCreate,
    ) -> Result<super::types::Timelog, anyhow::Error> {
        eprintln!("{}", serde_json::to_string(&log).unwrap());
        let logs: Vec<Timelog> = self.post_json_with_prefer_return("/timelogs", &log)?;
        logs.into_iter().next().context("No item in response")
    }

    fn timelog_update(
        &self,
        selector: TimelogQuery,
        patch: super::types::TimelogPatch,
    ) -> Result<Vec<Timelog>, anyhow::Error> {
        let mut query = build_timelog_query(&selector);
        query.set("select", "*");
        let path = format!("/timelogs?{}", query.to_query());
        self.patch_json_with_prefer_return(&path, &patch)
            .map_err(From::from)
    }
}
