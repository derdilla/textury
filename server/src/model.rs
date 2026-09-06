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
        #[sea_orm(has_many, via = "project_memberships")]
        pub member_of: HasMany<super::project::Entity>,
        /// ID of the user that invited this user, null for the initial account.
        pub invited_by: Option<i32>,
        pub is_admin: bool,
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
        pub title: String,
        pub owner_id: i32,
        #[sea_orm(belongs_to, from = "owner_id", to = "id")]
        pub owner: HasOne<super::user::Entity>,
        pub latest_compilation: Option<Vec<u8>>,
        pub last_modified: TimeDateTime,
        pub last_state: Option<Vec<u8>>,
        pub created_at: TimeDateTime,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod project_memberships {
    use sea_orm::entity::prelude::*;
    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "project_memberships")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub user_id: i32,
        #[sea_orm(primary_key)]
        pub project_id: String,
        #[sea_orm(belongs_to, from = "user_id", to = "id", on_delete = "Cascade")]
        pub user: HasOne<super::user::Entity>,
        #[sea_orm(belongs_to, from = "project_id", to = "id", on_delete = "Cascade")]
        pub project: HasOne<super::project::Entity>,
        /// Clients may use this to persist additional user-specific metadata.
        ///
        /// This is user specific and has no further impact.
        pub user_meta: Option<serde_json::Value>,
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
        pub issuer_id: Option<i32>,
        //#[sea_orm(belongs_to, from = "issuer_id", to = "id", on_delete = "Cascade")]
        //pub issuer: Option<super::user::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod asset {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "asset")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub project_id: String,
        #[sea_orm(primary_key)]
        pub asset_id: String,
        pub data: Vec<u8>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}
