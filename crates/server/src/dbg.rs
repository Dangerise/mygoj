use super::*;

pub async fn dbg() {
    user::register_user(shared::user::UserRegistration {
        email: "dangerise@qq.com".into(),
        password: "1234".into(),
        nickname: "Dangerise".into(),
    })
    .await
    .unwrap();
}
