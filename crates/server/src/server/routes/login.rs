use anyhow::bail;
use cookie::SameSite;
use http::StatusCode;
use maud::html;
use time::OffsetDateTime;
use wcgi::{Body, ResponseBuilder};

use crate::{
    logic::user::user_login,
    server::{
        prelude::{
            h2, h4, page, parse_form, response_html_ok, Context, Fragment, HandlerResult, Method,
            Request, Response,
        },
        response_not_found_html,
        ui::error_box,
        AUTH_COOKIE_NAME,
    },
};

pub fn handler_login(req: Request, ctx: &Context) -> HandlerResult {
    match *req.method() {
        Method::GET => Ok(handler_login_get(ctx)),
        Method::POST => match handler_login_submit(req, ctx) {
            Ok(r) => Ok(r),
            Err(err) => {
                let content = login_page(ctx, Some(err.to_string()));
                Ok(response_html_ok(content))
            }
        },
        _ => Ok(response_not_found_html()),
    }
}

pub fn handler_login_get(ctx: &Context) -> Response {
    response_html_ok(login_page(ctx, None))
}

pub fn build_auth_cookie(token: &str) -> cookie::Cookie {
    let mut c = cookie::Cookie::new(AUTH_COOKIE_NAME, token);
    c.set_path("/");
    c.set_secure(true);
    c.set_same_site(SameSite::Strict);
    c.set_http_only(true);
    // TODO: should be same as token?
    let expires = OffsetDateTime::now_utc().saturating_add(time::Duration::days(30));
    c.set_expires(expires);

    c
}

fn handler_login_submit(req: Request, ctx: &Context) -> HandlerResult {
    let data: LoginFormData = parse_form(req)?;

    if data.user.is_empty() {
        bail!("Must specify a username");
    }
    if data.password.is_empty() {
        bail!("must specify a password");
    }

    let (_user, token) = user_login(
        &ctx.db,
        &ctx.config.jwt_token_secret,
        &data.user,
        &data.password,
    )?;

    // Must set the auth cookie.

    let cookie = build_auth_cookie(&token);

    let res = ResponseBuilder::new()
        .status(StatusCode::SEE_OTHER)
        .header(http::header::LOCATION, "/")
        .header(http::header::SET_COOKIE, cookie.encoded().to_string())
        .body(Body::empty())
        .unwrap();

    Ok(res)
}

#[derive(serde::Deserialize, Debug)]
struct LoginFormData {
    user: String,
    password: String,
}

fn login_content(error: Option<String>) -> Fragment {
    let errmsg = if let Some(err) = error {
        error_box(err)
    } else {
        html! {}
    };

    html! {
        div.box {
            (h2("Timely - Time Tracker"))
        }

        div.box {
            (h4("Login"))

            form action="/login" method="post" {
                div.field {
                    label.label { "Username" }
                    input.input name="user" type="text" placeholder="username" {}
                }

                div.field {
                    label.label { "Password" }
                    input.input name="password" type="password" placeholder="******" {}
                }

                (errmsg)

                div.buttons {
                    button.button type="submit" { "Login" }
                    a class="button is-warning" href="/signup" { "Sign up" }
                }
            }
        }
    }
}

pub fn login_page(ctx: &Context, error: Option<String>) -> String {
    page(ctx, login_content(error))
}
