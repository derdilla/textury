use reqwest::StatusCode;
use server::api::types::{CreateAccountRequest, PrivateAccountInfo, User, UserInvite};

mod utils;

#[tokio::test]
async fn admins_can_manage_users() {
    let server = utils::test_server().await;
    let admin_client = reqwest::Client::builder()
        .cookie_store(true)
        .build().unwrap();
    let res = admin_client.post(format!("{}/user", server.url))
        .json(&CreateAccountRequest{
            username: "admin".to_string(),
            email: "contact@texturyadmin.info".to_string(),
            password: "verysecuresoftware.txt".to_string(),
            registration_token: server.token.clone(),
        })
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let res = admin_client.post(format!("{}/settings/users/invite", server.url)).send()
        .await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let invite: UserInvite = res.json().await.unwrap();

    let user_client = reqwest::Client::builder()
        .cookie_store(true)
        .build().unwrap();
    let res = user_client.post(format!("{}/user", server.url))
        .json(&CreateAccountRequest{
            username: "user".to_string(),
            email: "user@user.info".to_string(),
            password: "verysecuresoftware™".to_string(),
            registration_token: invite.registration_token,
        })
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let user_info: PrivateAccountInfo = user_client.get(format!("{}/user", server.url))
        .send().await.unwrap()
        .json().await.unwrap();
    assert_eq!(user_info.email, "user@user.info");
    assert!(!user_info.is_admin);

    // Find user in user list
    let res = admin_client.get(format!("{}/settings/users", server.url))
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let user_list: Vec<User> = res.json().await.unwrap();
    assert_eq!(user_list.len(), 2);
    assert_eq!(
        user_list.iter().find(|u| u.id == user_info.id).unwrap().username,
        user_info.username
    );

    // Update user
    let res = admin_client.put(format!("{}/settings/users/{}/isAdmin", server.url, user_info.id))
        .json(&true)
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let res = admin_client.put(format!("{}/settings/users/{}/username", server.url, user_info.id))
        .json(&"john")
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let res = admin_client.put(format!("{}/settings/users/{}/email", server.url, user_info.id))
        .json(&"john@user.info")
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let user_info: PrivateAccountInfo = user_client.get(format!("{}/user", server.url))
        .send().await.unwrap()
        .json().await.unwrap();
    assert_eq!(user_info.username, "john");
    assert_eq!(user_info.email, "john@user.info");
    assert!(user_info.is_admin);

    // Delete user
    let res = admin_client.delete(format!("{}/settings/users/{}", server.url, user_info.id))
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = user_client.get(format!("{}/user", server.url))
        .send().await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
