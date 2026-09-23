use server::api::types::CreateAccountRequest;

use crate::utils::client;

mod utils;

#[tokio::test]
async fn admin_can_register() {
    let server = utils::test_server().await;
    let client = client::Client::new(&server.url);
    let res = client.post_user(&CreateAccountRequest{
        email: "contact@texturyadmin.info".to_string(),
        password: "verysecuresoftware.txt".to_string(),
        registration_token: server.token.clone(),
        username: "admin".to_string(),
    }).await.unwrap();
    assert_eq!(res.email, "contact@texturyadmin.info");
    assert_eq!(res.username, "admin");
    assert_eq!(res.is_admin, true);
    assert!(res.settings.is_empty());
}
