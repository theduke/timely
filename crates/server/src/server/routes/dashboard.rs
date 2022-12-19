use maud::html;
use time::format_description::well_known::Rfc3339;

use crate::{
    db::{user_active_timelogs, user_finished_timelogs, Db},
    server::{
        prelude::{h2, page, response_html_ok, Context, Fragment, HandlerResult, Method, Request},
        response_not_found_html,
        ui::{error_box, util::renderiter},
    },
};

pub fn handler_dashboard(req: Request, ctx: &Context) -> HandlerResult {
    match *req.method() {
        Method::GET => {
            let content = build_dashboard(ctx, None)?;
            Ok(response_html_ok(page(ctx, content)))
        }
        _ => Ok(response_not_found_html()),
    }
}

pub fn build_dashboard(ctx: &Context, error: Option<String>) -> Result<Fragment, anyhow::Error> {
    let user = ctx.require_user()?;

    let unfinished = ctx.db.timelogs(user_active_timelogs(user.id))?;

    let errmsg = error.map(error_box).unwrap_or_else(|| html! {});

    let active_logs = if !unfinished.is_empty() {
        let multi_warning = if unfinished.len() > 1 {
            html! {
                p class="notification is-warning" {
                    "Multiple trackers active!"
                }
            }
        } else {
            html! {}
        };

        let items = unfinished.iter().map(|item| {
            html! {
                div.box {
                    div {
                        b {
                            (item.title)
                        }
                    }

                    div {
                        "Started: "
                        (item.started_at.format(&Rfc3339).unwrap())
                    }

                    div.buttons {
                        form action="/timelog/finish" method="post" {
                            input name="timelog_id" value=(item.id) type="hidden" {}
                            button.button {
                                "Finish"
                            }
                        }
                    }

                }
            }
        });

        html! {
            (multi_warning)
            (renderiter(items))
        }
    } else {
        html! {
            div.box {
                (log_start_form())
            }
        }
    };

    let finished_logs = ctx.db.timelogs(user_finished_timelogs(user.id))?;
    let old_logs = if finished_logs.is_empty() {
        html! {
            div class="notification is-warning" {
                "No logs created yet."
            }
        }
    } else {
        let items = finished_logs.iter().map(|item| {
            let started = item.started_at.format(&Rfc3339).unwrap();
            let finished = item
                .finished_at()
                .and_then(|t| t.format(&Rfc3339).ok())
                .unwrap_or_default();

            html! {
                div.box {
                    div {
                        (item.title)
                    }

                    div class="is-flex" style="gap: 1rem" {
                        div {
                            b { "Started: " }
                            (started)
                        }
                        div {
                            b { "Finished: " }
                            (finished)
                        }
                    }
                }
            }
        });

        html! {
            div {
                (renderiter(items))
            }
        }
    };

    let out = html! {
        div.container {
            (h2("DASHBOARD"))
            (errmsg)
            (active_logs)
            hr {}
            (old_logs)
        }
    };
    Ok(out)
}

fn log_start_form() -> Fragment {
    html! {
        form action="/timelog/start" method="post" {
            div.field {
                label.label { "Title" }
                input.input name="title" type="text" placeholder="..." {}
            }

            div.buttons {
                button.button type="submit" { "Start" }
            }
        }
    }
}
