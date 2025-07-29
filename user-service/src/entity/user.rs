use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Debug, PartialEq, PartialOrd, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(unique, indexed)]
    pub user_id: i64,
    pub total_credits: i64,
    pub credits_remaining: i64,
    pub subscription_status: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::session::Entity")]
    Session,
}
impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
