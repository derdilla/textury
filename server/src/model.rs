mod user {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "user")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        #[sea_orm(unique)]
        pub email: String,
        #[sea_orm(has_many, via = "project_memberships")]
        pub member_of: HasMany<suprt::project::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod project {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "project")]
    pub struct Model {
        #[sea_orm(primary_key, unique)]
        pub id: String,
        pub owner_id: i32,
        #[sea_orm(belongs_to, from = "owner_id", to = "id")]
        pub owner: HasOne<super::user::Entity>,
        #[sea_orm(has_many, via = "project_memberships")]
        pub members: HasMany<suprt::user::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/* TODO: implement storing data
mod Directory {

    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "project")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: u64,
        pub parent_id: Option<u64>,
        pub title: String,

    }

    impl ActiveModelBehavior for ActiveModel {}
}*/
