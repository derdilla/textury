use reqwest::StatusCode;
use server::api::types::{CreateAccountRequest, PrivateAccountInfo};

mod utils;

#[tokio::test]
async fn admin_can_register() {
    let server = utils::test_server().await;
    let client = reqwest::Client::new();
    let res = client.post(format!("{}/user", server.url))
        .json(&CreateAccountRequest{
            email: "contact@texturyadmin.info".to_string(),
            password: "verysecuresoftware.txt".to_string(),
            registration_token: server.token.clone(),
            username: "admin".to_string(),
        })
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let res: PrivateAccountInfo = res.json().await.unwrap();
    assert_eq!(res.email, "contact@texturyadmin.info");
    assert_eq!(res.username, "admin");
    assert_eq!(res.is_admin, true);
    assert!(res.settings.is_empty());
}
