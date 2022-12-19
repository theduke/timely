use self::types::{
    Order, Timelog, TimelogCreate, TimelogFilter, TimelogId, TimelogOrder, TimelogPatch,
    TimelogQuery, User, UserCreate, UserFilter, UserId, UserQuery,
};

pub mod client_supabase;
pub mod types;

pub trait Db {
    fn user(&self, filter: UserFilter) -> Result<Option<User>, anyhow::Error>;
    fn users(&self, query: UserQuery) -> Result<Vec<User>, anyhow::Error>;
    fn user_create(&self, user: UserCreate) -> Result<User, anyhow::Error>;

    fn timelogs(&self, query: TimelogQuery) -> Result<Vec<Timelog>, anyhow::Error>;
    fn timelog_create(&self, log: TimelogCreate) -> Result<Timelog, anyhow::Error>;
    fn timelog_update(
        &self,
        selector: TimelogQuery,
        patch: TimelogPatch,
    ) -> Result<Vec<Timelog>, anyhow::Error>;
}

pub fn user_active_timelogs(user_id: UserId) -> TimelogQuery {
    TimelogQuery {
        filter: Some(TimelogFilter::UserId(user_id).and(TimelogFilter::IsFinished(false))),
        limit: 100,
        offset: 0,
        order: vec![Order::desc(TimelogOrder::StartedAt)],
    }
}

pub fn user_finished_timelogs(user_id: UserId) -> TimelogQuery {
    TimelogQuery {
        filter: Some(TimelogFilter::UserId(user_id).and(TimelogFilter::IsFinished(true))),
        limit: 100,
        offset: 0,
        order: vec![Order::desc(TimelogOrder::StartedAt)],
    }
}

pub fn timelog_by_id(id: TimelogId) -> TimelogQuery {
    TimelogQuery {
        filter: Some(TimelogFilter::Id(id)),
        limit: 1,
        offset: 0,
        order: vec![],
    }
}
