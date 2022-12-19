use anyhow::Context as _;
use cookie::{Cookie, CookieJar};
use http::{Method, StatusCode};
use time::OffsetDateTime;
use wcgi::{Body, Request, Response, ResponseBuilder, WcgiError};

use crate::db::{client_supabase::SupaDb, types::User};

use self::{routes::login::build_auth_cookie, ui::error_page};

mod routes;
pub mod ui;

pub mod prelude {
    pub use http::{Method, StatusCode};
    pub use wcgi::{Request, Response, ResponseBuilder};

    pub use super::{
        response_html, response_html_ok,
        ui::{self, h2, h4, page, Fragment},
        Context, HandlerResult,
    };

    use anyhow::anyhow;

    pub fn parse_form<T: serde::de::DeserializeOwned>(req: Request) -> Result<T, anyhow::Error> {
        let body = req
            .into_body()
            .read_to_vec()
            .map_err(|err| anyhow!("Could not read request body: {err}"))?;
        let data: T = serde_urlencoded::from_bytes(&body)?;
        Ok(data)
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub supabase_endpoint: String,
    pub supabase_api_key: String,
    /// JWT token secret for encoding and decoding.
    pub jwt_token_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let supabase_endpoint = std::env::var("SUPABASE_ENDPOINT")
            .ok()
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .context("Missing required env var SUPABASE_ENDPOINT")?;
        let supabase_api_key = std::env::var("SUPABASE_KEY")
            .ok()
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .context("Missing required env var SUPABASE_KEY")?;

        let jwt_token_secret = std::env::var("TIMELY_TOKEN_SECRET")
            .ok()
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .context("Missing required env var TIMELY_TOKEN_SECRET")?;

        Ok(Self {
            supabase_endpoint,
            supabase_api_key,
            jwt_token_secret,
        })
    }
}

#[derive(Clone)]
pub struct Context {
    config: Config,
    db: SupaDb,
    user: Option<User>,
}

impl Context {
    pub fn new(config: Config) -> Result<Self, anyhow::Error> {
        let db = SupaDb::new(
            config.supabase_endpoint.clone(),
            config.supabase_api_key.clone(),
        )
        .expect("Invalid configuration");

        let c = Context {
            config,
            db,
            user: None,
        };

        Ok(c)
    }

    pub fn require_user(&self) -> Result<&User, anyhow::Error> {
        self.user.as_ref().context("expected a user in the context")
    }
}

pub type HandlerResult = Result<Response, anyhow::Error>;

const AUTH_COOKIE_NAME: &str = "timelytoken";

pub fn handler(ctx: &Context, req: Request) -> Result<Response, WcgiError> {
    let uri = req.uri();
    let path = uri.path().to_string();
    let path_parts = path
        .strip_prefix('/')
        .unwrap_or(uri.path())
        .split('/')
        .collect::<Vec<_>>();

    let cookies = get_cookies(&req);
    let user = match cookies.get(AUTH_COOKIE_NAME) {
        Some(c) => {
            match crate::logic::user::load_user_for_token(
                &ctx.db,
                &ctx.config.jwt_token_secret,
                c.value(),
            ) {
                Ok(u) => Some(u),
                Err(err) => {
                    eprintln!("invalid token: {err}");

                    // Invalid token - must reset the cookie.
                    let res = response_reset_auth_cookies();
                    return Ok(res);
                }
            }
        }
        None => None,
    };

    let ctx = Context {
        user,
        ..ctx.clone()
    };

    let res = if ctx.user.is_none() {
        match (path_parts.as_slice(), req.method().clone()) {
            (["signup"], _) => routes::signup::handler_signup(req, &ctx),
            _ => routes::login::handler_login(req, &ctx),
        }
    } else {
        match (path_parts.as_slice(), req.method().clone()) {
            ([], Method::GET) => routes::dashboard::handler_dashboard(req, &ctx),
            (["timelog", "start"], Method::POST) => routes::timelog_start::handler(req, &ctx),
            (["timelog", "finish"], Method::POST) => routes::timelog_finish::handler(req, &ctx),
            (["user", "logout"], Method::POST) => Ok(response_reset_auth_cookies()),
            (_, Method::GET) => routes::dashboard::handler_dashboard(req, &ctx),
            (_, method) => {
                eprintln!("path not found: method={} parts={:?}", method, path_parts);
                Ok(response_not_found_html())
            }
        }
    };

    let res: Result<Response, WcgiError> = match res {
        Ok(r) => Ok(r),
        Err(err) => {
            eprintln!("ERROR: {err:?}");
            Ok(response_html(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_page(&ctx, err),
            ))
            // }
        }
    };
    res
}

fn response_reset_auth_cookies() -> Response {
    let mut res = response_redirect_tmp("/");
    let mut c = build_auth_cookie("");
    c.set_expires(OffsetDateTime::now_utc() - time::Duration::days(30));
    c.set_value("");
    res.headers_mut().append(
        http::header::SET_COOKIE,
        c.encoded().to_string().parse().unwrap(),
    );
    res
}

pub fn response_html_ok(body: impl Into<String>) -> Response {
    ResponseBuilder::new()
        .status(StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "text/html")
        .body(Body::new_text(body.into()))
        .unwrap()
}

pub fn response_html(status: StatusCode, body: impl Into<String>) -> Response {
    ResponseBuilder::new()
        .status(status)
        .header(http::header::CONTENT_TYPE, "text/html")
        .body(Body::new_text(body.into()))
        .unwrap()
}

fn response_redirect_tmp(to: &str) -> Response {
    ResponseBuilder::new()
        .status(StatusCode::SEE_OTHER)
        .header(http::header::LOCATION, to)
        .body(Body::empty())
        .unwrap()
}

// fn response_html_from_res<B: Into<String>>(
//     ctx: &Context,
//     res: Result<B, anyhow::Error>,
// ) -> Response {
//     match res {
//         Ok(b) => response_html(StatusCode::OK, b.into()),
//         Err(err) => response_html(StatusCode::INTERNAL_SERVER_ERROR, ui::error_page(&ctx, er)),
//     }
// }

fn response_not_found_html() -> Response {
    ResponseBuilder::new()
        .status(StatusCode::NOT_FOUND)
        .header(http::header::CONTENT_TYPE, "text/html")
        .body(Body::new_text(ui::page_not_found()))
        .unwrap()
}

fn get_cookies(req: &Request) -> CookieJar {
    let mut jar = CookieJar::new();
    for header in req.headers().get_all(http::header::COOKIE) {
        let res = header
            .to_str()
            .ok()
            .and_then(|s| Cookie::parse_encoded(s.to_string()).ok());
        if let Some(c) = res {
            jar.add_original(c);
        }
    }

    jar
}
