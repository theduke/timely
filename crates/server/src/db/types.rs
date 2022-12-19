use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub type UserId = u64;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    #[serde(
        serialize_with = "time::serde::rfc3339::serialize",
        deserialize_with = "time::serde::rfc3339::deserialize"
    )]
    pub created_at: time::OffsetDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserCreate {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserTag {
    pub id: u64,
    pub user_id: UserId,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    #[serde(
        serialize_with = "time::serde::rfc3339::serialize",
        deserialize_with = "time::serde::rfc3339::deserialize"
    )]
    pub created_at: time::OffsetDateTime,
    #[serde(
        serialize_with = "time::serde::rfc3339::serialize",
        deserialize_with = "time::serde::rfc3339::deserialize"
    )]
    pub updated_at: time::OffsetDateTime,
}

pub type TimelogId = u64;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Timelog {
    pub id: TimelogId,
    pub user_id: UserId,
    pub title: String,
    pub description: Option<String>,

    #[serde(
        serialize_with = "time::serde::rfc3339::serialize",
        deserialize_with = "time::serde::rfc3339::deserialize"
    )]
    pub created_at: time::OffsetDateTime,
    #[serde(
        serialize_with = "time::serde::rfc3339::serialize",
        deserialize_with = "time::serde::rfc3339::deserialize"
    )]
    pub started_at: time::OffsetDateTime,
    // TODO: better serde integration, as above.
    pub finished_at: Option<String>,
}

impl Timelog {
    pub fn finished_at(&self) -> Option<OffsetDateTime> {
        self.finished_at
            .as_ref()
            .and_then(|t| OffsetDateTime::parse(t, &Rfc3339).ok())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimelogCreate {
    pub user_id: UserId,
    pub title: String,
    pub description: Option<String>,
    #[serde(
        serialize_with = "time::serde::rfc3339::serialize",
        deserialize_with = "time::serde::rfc3339::deserialize"
    )]
    pub created_at: time::OffsetDateTime,
    #[serde(
        serialize_with = "time::serde::rfc3339::serialize",
        deserialize_with = "time::serde::rfc3339::deserialize"
    )]
    pub started_at: time::OffsetDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimelogPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
}

#[derive(Clone, Debug)]
pub enum TimelogFilter {
    Id(TimelogId),
    UserId(UserId),
    IsFinished(bool),
    And(Vec<Self>),
}

impl TimelogFilter {
    pub fn and(self, other: Self) -> Self {
        Self::And(vec![self, other])
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Direction {
    Asc,
    Desc,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Order<T> {
    pub expr: T,
    pub direction: Direction,
}

impl<T> Order<T> {
    pub fn new(expr: impl Into<T>, direction: Direction) -> Self {
        Self {
            expr: expr.into(),
            direction,
        }
    }

    pub fn asc(expr: impl Into<T>) -> Self {
        Self::new(expr, Direction::Asc)
    }

    pub fn desc(expr: impl Into<T>) -> Self {
        Self::new(expr, Direction::Desc)
    }
}

#[derive(Clone, Debug)]
pub enum TimelogOrder {
    Id,
    StartedAt,
}

#[derive(Clone, Debug)]
pub struct TimelogQuery {
    pub filter: Option<TimelogFilter>,
    pub limit: u64,
    pub offset: u64,
    pub order: Vec<Order<TimelogOrder>>,
}

impl TimelogQuery {
    pub fn new() -> Self {
        Self {
            filter: None,
            limit: 50,
            offset: 0,
            order: vec![],
        }
    }

    pub fn new_single_id(id: TimelogId) -> Self {
        Self {
            filter: Some(TimelogFilter::Id(id)),
            limit: 1,
            ..Self::new()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimelogUserTag {
    pub user_tag_id: u64,
    pub timelog_id: TimelogId,
}

#[derive(Clone, Debug)]
pub enum UserFilter {
    Id(UserId),
    Name(String),
}

#[derive(Clone, Debug)]
pub struct UserQuery {
    pub filter: Option<UserFilter>,
    pub limit: u64,
    pub offset: u64,
}
