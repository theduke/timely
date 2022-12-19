use anyhow::{bail, Context as _};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::{
    db::{
        timelog_by_id,
        types::{Timelog, TimelogPatch, TimelogQuery},
        Db,
    },
    server::prelude::{page, parse_form, response_html_ok, Context, HandlerResult, Request},
};

#[derive(serde::Deserialize, Clone)]
struct FinishFormData {
    timelog_id: u64,
}

pub fn handler(req: Request, ctx: &Context) -> HandlerResult {
    let err = match try_finish(req, ctx) {
        Ok(_) => None,
        Err(err) => Some(err.to_string()),
    };
    let content = super::dashboard::build_dashboard(ctx, err)?;
    Ok(response_html_ok(page(ctx, content)))
}

pub fn try_finish(req: Request, ctx: &Context) -> Result<Timelog, anyhow::Error> {
    let data: FinishFormData = parse_form(req)?;

    let _user = ctx.require_user()?;

    let log = ctx
        .db
        .timelogs(timelog_by_id(data.timelog_id))?
        .into_iter()
        .next()
        .context("Timelog not found")?;
    if log.finished_at.is_some() {
        bail!("Log entry already closed");
    }

    let now = OffsetDateTime::now_utc();

    let selector = TimelogQuery::new_single_id(data.timelog_id);
    let patch = TimelogPatch {
        title: None,
        description: None,
        finished_at: Some(now.format(&Rfc3339).unwrap()),
    };
    let out = ctx
        .db
        .timelog_update(selector, patch)?
        .into_iter()
        .next()
        .context("not found")?;

    assert!(out.finished_at.is_some());

    Ok(out)
}
