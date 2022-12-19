use anyhow::bail;
use time::OffsetDateTime;

use crate::{
    db::{
        types::{Timelog, TimelogCreate},
        user_active_timelogs, Db,
    },
    server::prelude::{page, parse_form, response_html_ok, Context, HandlerResult, Request},
};

#[derive(serde::Deserialize, Clone)]
struct StartFormData {
    title: String,
}

pub fn handler(req: Request, ctx: &Context) -> HandlerResult {
    let err = match try_start(req, ctx) {
        Ok(_) => None,
        Err(err) => Some(err.to_string()),
    };
    let content = super::dashboard::build_dashboard(ctx, err)?;
    Ok(response_html_ok(page(ctx, content)))
}

pub fn try_start(req: Request, ctx: &Context) -> Result<Timelog, anyhow::Error> {
    let data: StartFormData = parse_form(req)?;

    let title = data.title.trim().to_string();
    if title.is_empty() {
        bail!("Title may not be empty");
    }

    let user = ctx.require_user()?;
    let active = ctx.db.timelogs(user_active_timelogs(user.id))?;
    if !active.is_empty() {
        bail!("Other running tasks - finish them first!");
    }

    let now = OffsetDateTime::now_utc();

    let create = TimelogCreate {
        user_id: user.id,
        title,
        description: None,
        created_at: now.clone(),
        started_at: now,
    };
    ctx.db.timelog_create(create)
}
