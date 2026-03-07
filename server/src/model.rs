pub use user::Entity as User;
pub use project::Entity as Project;

pub mod user {
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
        pub password_hash: String,
        pub password_salt: String,
        #[sea_orm(has_many, via = "project_memberships")]
        pub member_of: HasMany<super::project::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod project {
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
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod project_memberships {
    use sea_orm::entity::prelude::*;
    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "project_memberships")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub user_id: i32,
        #[sea_orm(primary_key, auto_increment = false)]
        pub project_id: String,
        #[sea_orm(
            belongs_to,
            from = "user_id",
            to = "id"
        )]
        pub user: HasOne<super::user::Entity>,
        #[sea_orm(
            belongs_to,
            from = "project_id",
            to = "id"
        )]
        pub project: HasOne<super::project::Entity>,

    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod registration_tokens {
    use sea_orm::entity::prelude::*;
    use time::OffsetDateTime;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "registration_tokens")]
    pub struct Model {
        #[sea_orm(primary_key, unique)]
        pub token: String,
        pub expires: OffsetDateTime,
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
        pub id: i32,
        pub parent_id: Option<i32>,
        pub title: String,

    }

    impl ActiveModelBehavior for ActiveModel {}
}*/
