use anyhow::bail;
use http::StatusCode;
use maud::html;
use time::OffsetDateTime;
use wcgi::{Body, ResponseBuilder};

use crate::server::{
    prelude::{
        h2, h4, page, parse_form, response_html_ok, Context, Fragment, HandlerResult, Method,
        Request, Response,
    },
    response_not_found_html,
    ui::error_box,
    AUTH_COOKIE_NAME,
};

pub fn handler_signup(req: Request, ctx: &Context) -> HandlerResult {
    match *req.method() {
        Method::GET => Ok(handler_signup_get(ctx)),
        Method::POST => match handler_signup_submit(req, ctx) {
            Ok(r) => Ok(r),
            Err(err) => {
                let content = signup_page(ctx, Some(err.to_string()));
                Ok(response_html_ok(content))
            }
        },
        _ => Ok(response_not_found_html()),
    }
}

pub fn handler_signup_get(ctx: &Context) -> Response {
    response_html_ok(signup_page(ctx, None))
}

fn handler_signup_submit(req: Request, ctx: &Context) -> HandlerResult {
    let data: LoginFormData = parse_form(req)?;

    if data.username.is_empty() {
        bail!("Must specify a username");
    }
    if data.email.is_empty() {
        bail!("Must specify a username");
    }
    if data.password.is_empty() {
        bail!("must specify a password");
    }

    let data = crate::logic::user::Signup {
        username: data.username,
        email: data.email,
        password: data.password,
    };

    let (_user, token) =
        crate::logic::user::user_signup_and_login(&ctx.db, &ctx.config.jwt_token_secret, data)?;

    // Must set the auth cookie.
    let mut authcookie = cookie::Cookie::new(AUTH_COOKIE_NAME, token);
    // TODO: should be same as token?
    let expires = OffsetDateTime::now_utc().saturating_add(time::Duration::days(30));
    authcookie.set_expires(expires);

    let res = ResponseBuilder::new()
        .status(StatusCode::SEE_OTHER)
        .header(http::header::LOCATION, "/")
        .header(http::header::SET_COOKIE, authcookie.encoded().to_string())
        .body(Body::empty())
        .unwrap();

    Ok(res)
}

#[derive(serde::Deserialize, Debug)]
struct LoginFormData {
    username: String,
    password: String,
    email: String,
}

fn signup_content(error: Option<String>) -> Fragment {
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
            form method="post" action="/signup" {
                (h4("Sign up"))

                div.field {
                    label.label { "Username" }
                    input.input name="username" type="text" placeholder="username" {}
                }

                div.field {
                    label.label { "Email" }
                    input.input name="email" type="text" placeholder="my@email.com" {}
                }

                div.field {
                    label.label { "Password" }
                    input.input name="password" type="password" placeholder="******" {}
                }

                (errmsg)

                div.buttons {
                    button.button type="submit" { "Sign up" }
                    a class="button" href="/login" { "Log in" }
                }
            }
        }

    }
}

pub fn signup_page(ctx: &Context, error: Option<String>) -> String {
    page(ctx, signup_content(error))
}
