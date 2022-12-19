pub mod util;

use maud::{html, Render};

use self::util::navbar;

use super::Context;

pub type Fragment = maud::PreEscaped<String>;

pub fn h2(content: impl Render) -> Fragment {
    html! {
        h2 class="title is-2" {
            (content)
        }
    }
}

// pub fn h3(content: impl Render) -> Fragment {
//     html! {
//         h2 class="title is-3" {
//             (content)
//         }
//     }
// }

pub fn h4(content: impl Render) -> Fragment {
    html! {
        h2 class="title is-4" {
            (content)
        }
    }
}

pub fn error_box(e: impl maud::Render) -> Fragment {
    html! {
        p.notification.is-danger {
            (e)
        }
    }
}

pub fn page(ctx: &Context, content: Fragment) -> String {
    let navbar = if let Some(user) = &ctx.user {
        navbar(ctx, user)
    } else {
        html! {}
    };

    html!{
        html {
            head {
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css" { }
            }

            body {
                (navbar)
                (content)
            }
        }
    }.into_string()
}

pub fn error_page(ctx: &Context, error: anyhow::Error) -> String {
    let msg = error.to_string();
    let content = html! {
        (error_box(msg))
    };
    page(ctx, content)
}

pub fn page_not_found() -> Fragment {
    html! {
        div {
            p.alert.is-warning {
                "Not found"
            }
        }
    }
}
