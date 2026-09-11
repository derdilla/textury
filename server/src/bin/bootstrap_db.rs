#[tokio::main]
async fn main() {
    let conn = sqlx::SqlitePool::connect("sqlite://./db.sqlite?mode=rwc").await.unwrap();
    sqlx::migrate!("./migrations").run(&conn).await.unwrap();
}
