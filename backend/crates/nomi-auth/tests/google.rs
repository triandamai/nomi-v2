use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use nomi_auth::google::{self as google_sign_in, Finished, GoogleSignInConfig, Purpose, SignInError};

/// Every Google endpoint on the mock server.
fn config_at(base: &str) -> GoogleSignInConfig {
    GoogleSignInConfig {
        client_id: "client-id".into(),
        client_secret: "client-secret".into(),
        redirect_uri: "http://app.test/auth/google/callback".into(),
        auth_url: format!("{base}/o/oauth2/v2/auth"),
        token_url: format!("{base}/token"),
        userinfo_url: format!("{base}/v1/userinfo"),
    }
}

/// Google answers a code with this person: `sub`, `email`, and whether the email is verified.
async fn google_person(server: &MockServer, code: &str, sub: &str, email: &str, verified: bool) {
    let token = format!("tok-{code}");
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains(format!("code={code}")))
        .and(body_string_contains("redirect_uri=http%3A%2F%2Fapp.test%2Fauth%2Fgoogle%2Fcallback"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "access_token": token, "expires_in": 3600 })))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/userinfo"))
        .and(header("authorization", format!("Bearer {token}").as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "sub": sub, "email": email, "email_verified": verified, "name": "Ana Putri", "given_name": "Ana"
        })))
        .mount(server)
        .await;
}

async fn sign_in(pool: &PgPool, server: &MockServer, purpose: Purpose, code: &str) -> Result<Finished, SignInError> {
    let config = config_at(&server.uri());
    let url = google_sign_in::start(pool, &config, purpose).await.unwrap();
    let url = reqwest::Url::parse(&url).unwrap();
    assert!(url.query_pairs().any(|(k, v)| k == "scope" && v == "openid email profile"), "sign-in only asks who you are");
    let state = url.query_pairs().find(|(k, _)| k == "state").unwrap().1.to_string();
    google_sign_in::finish(pool, &reqwest::Client::new(), &config, code, &state).await
}

async fn password_user(pool: &PgPool, email: &str) -> Uuid {
    nomi_auth::registration::register_user(pool, email, "correct-password", nomi_auth::registration::OrgMode::Create { name: "Acme".into() })
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_first_google_sign_in_creates_the_account_and_the_next_one_signs_in(pool: PgPool) {
    let server = MockServer::start().await;
    google_person(&server, "c1", "sub-ana", "Ana@Gmail.example", true).await;
    google_person(&server, "c2", "sub-ana", "ana@gmail.example", true).await;

    let first = sign_in(&pool, &server, Purpose::SignIn { invite_code: None }, "c1").await.unwrap();
    let Finished::SignedIn { user_id, email, is_new: true } = first else { panic!("expected a new account, got {first:?}") };
    assert_eq!(email, "ana@gmail.example");

    let (org, name, password_login): (String, Option<String>, bool) = sqlx::query_as(
        "SELECT o.name, p.display_name, wc.password_login FROM memberships m JOIN organizations o ON o.id = m.org_id \
         JOIN web_credentials wc ON wc.user_id = m.user_id LEFT JOIN user_profiles p ON p.user_id = m.user_id WHERE m.user_id = $1",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((org.as_str(), name.as_deref(), password_login), ("Ana's space", Some("Ana Putri"), false));
    assert!(nomi_auth::login::issue_access_token(&pool, user_id, "secret").await.is_ok());

    let again = sign_in(&pool, &server, Purpose::SignIn { invite_code: None }, "c2").await.unwrap();
    assert_eq!(again, Finished::SignedIn { user_id, email: "ana@gmail.example".into(), is_new: false });
}

#[sqlx::test(migrations = "../../migrations")]
async fn google_with_the_same_verified_email_signs_in_to_the_existing_password_account(pool: PgPool) {
    let server = MockServer::start().await;
    let existing = password_user(&pool, "ana@gmail.example").await;
    google_person(&server, "c1", "sub-ana", "ana@gmail.example", true).await;

    let result = sign_in(&pool, &server, Purpose::SignIn { invite_code: None }, "c1").await.unwrap();

    assert_eq!(result, Finished::SignedIn { user_id: existing, email: "ana@gmail.example".into(), is_new: false });
    let methods = google_sign_in::methods(&pool, existing).await.unwrap();
    assert!(methods.password, "the password still works");
    assert_eq!(methods.google_email.as_deref(), Some("ana@gmail.example"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unverified_google_email_never_takes_over_or_creates_an_account(pool: PgPool) {
    let server = MockServer::start().await;
    password_user(&pool, "ana@gmail.example").await;
    google_person(&server, "c1", "sub-mallory", "ana@gmail.example", false).await;

    let result = sign_in(&pool, &server, Purpose::SignIn { invite_code: None }, "c1").await;

    assert_eq!(result, Err(SignInError::EmailNotVerified));
    let identities: i64 = sqlx::query_scalar("SELECT count(*) FROM google_identities").fetch_one(&pool).await.unwrap();
    assert_eq!(identities, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn linking_adds_google_to_the_signed_in_account_but_never_steals_anothers(pool: PgPool) {
    let server = MockServer::start().await;
    let ana = password_user(&pool, "ana@example.com").await;
    let budi = password_user(&pool, "budi@example.com").await;
    google_person(&server, "c1", "sub-ana", "ana.putri@gmail.example", true).await;
    google_person(&server, "c2", "sub-ana", "ana.putri@gmail.example", true).await;

    let linked = sign_in(&pool, &server, Purpose::Link { user_id: ana }, "c1").await.unwrap();
    assert_eq!(linked, Finished::Linked { user_id: ana, email: "ana.putri@gmail.example".into() });

    let stolen = sign_in(&pool, &server, Purpose::Link { user_id: budi }, "c2").await;
    assert_eq!(stolen, Err(SignInError::LinkedElsewhere));
}

#[sqlx::test(migrations = "../../migrations")]
async fn google_can_only_be_unlinked_when_a_password_still_signs_in(pool: PgPool) {
    let server = MockServer::start().await;
    google_person(&server, "c1", "sub-new", "new@gmail.example", true).await;
    google_person(&server, "c2", "sub-ana", "ana.putri@gmail.example", true).await;
    let Finished::SignedIn { user_id: google_only, .. } = sign_in(&pool, &server, Purpose::SignIn { invite_code: None }, "c1").await.unwrap() else {
        panic!()
    };
    let ana = password_user(&pool, "ana@example.com").await;
    sign_in(&pool, &server, Purpose::Link { user_id: ana }, "c2").await.unwrap();

    assert!(!google_sign_in::unlink(&pool, google_only).await.unwrap(), "would lock them out");
    assert!(google_sign_in::unlink(&pool, ana).await.unwrap());
    assert_eq!(google_sign_in::methods(&pool, ana).await.unwrap().google_email, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_or_reused_state_is_refused(pool: PgPool) {
    let server = MockServer::start().await;
    let config = config_at(&server.uri());
    let result = google_sign_in::finish(&pool, &reqwest::Client::new(), &config, "code", "made-up").await;
    assert_eq!(result, Err(SignInError::Expired));
}
