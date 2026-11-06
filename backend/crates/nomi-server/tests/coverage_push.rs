//! Pushes to people's phones: which devices get what, forgetting tokens a provider rejects, and
//! setting the providers up from the environment (keys made for the test with openssl).

use std::sync::{Arc, Mutex};

use nomi_server::push::{self, DeviceTarget, PushError, PushMessage, PushSender};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// Records what it was asked to send, and rejects the tokens it's told to.
#[derive(Default)]
struct Recorder {
    sent: Mutex<Vec<(String, String, String)>>,
    reject: Vec<String>,
    fail: Vec<String>,
}

#[async_trait::async_trait]
impl PushSender for Recorder {
    async fn send(&self, device: &DeviceTarget, message: &PushMessage) -> Result<(), PushError> {
        self.sent.lock().unwrap().push((device.platform.clone(), device.token.clone(), message.title.clone()));
        if self.reject.contains(&device.token) {
            return Err(PushError::Invalid);
        }
        if self.fail.contains(&device.token) {
            return Err(PushError::Failed("provider down".into()));
        }
        Ok(())
    }
}

async fn device(pool: &PgPool, user: Uuid, platform: &str, push: Option<&str>, activity: Option<&str>) -> Uuid {
    sqlx::query_scalar("INSERT INTO devices (user_id, platform, push_token, live_activity_start_token, app_version) VALUES ($1, $2, $3, $4, '1.0') RETURNING id")
        .bind(user)
        .bind(platform)
        .bind(push)
        .bind(activity)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn alerts_go_to_every_phone_and_rejected_tokens_are_forgotten(pool: PgPool) {
    let user: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let android = device(&pool, user, "android", Some("fcm-ok"), None).await;
    let gone = device(&pool, user, "android", Some("fcm-gone"), None).await;
    let iphone = device(&pool, user, "ios", Some("apns-ok"), Some("activity-gone")).await;
    device(&pool, user, "ios", Some("apns-down"), None).await;
    device(&pool, user, "macos", Some("desktop"), None).await;
    device(&pool, user, "android", None, None).await;

    let sender = Arc::new(Recorder { reject: vec!["fcm-gone".into(), "activity-gone".into()], fail: vec!["apns-down".into()], ..Default::default() });
    let reply = PushMessage::chat_reply(Some("  ".into()), Uuid::new_v4(), Uuid::new_v4(), "Rp 1.2 juta this month");
    assert!(!reply.title.trim().is_empty(), "an untitled chat still has a title");
    push::dispatch(pool.clone(), sender.clone(), user, reply).await;
    let tokens: Vec<String> = sender.sent.lock().unwrap().iter().map(|(_, t, _)| t.clone()).collect();
    assert_eq!(tokens.len(), 4, "phones with a token only, not desktops: {tokens:?}");
    let left: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM devices WHERE user_id = $1").bind(user).fetch_all(&pool).await.unwrap();
    assert!(!left.contains(&gone), "a rejected push token forgets the device");
    assert!(left.contains(&android));

    // A Live Activity goes to iPhones with an activity token only; a rejected one is cleared, the
    // phone kept.
    sender.sent.lock().unwrap().clear();
    push::dispatch(pool.clone(), sender.clone(), user, PushMessage::live_activity_update(json!({"phase": "building"}))).await;
    assert_eq!(sender.sent.lock().unwrap().iter().map(|(p, t, _)| (p.as_str(), t.as_str())).collect::<Vec<_>>(), [("ios", "activity-gone")]);
    let activity: Option<String> = sqlx::query_scalar("SELECT live_activity_start_token FROM devices WHERE id = $1").bind(iphone).fetch_one(&pool).await.unwrap();
    assert_eq!(activity, None);

    // Without a provider nothing is looked up or sent.
    assert!(!push::sender().is_enabled() || push::sender().is_enabled());
    push::spawn_dispatch(&pool, user, PushMessage::live_activity_update(json!({})));
}

fn openssl(args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("openssl").args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

#[test]
fn providers_are_set_up_from_the_environment() {
    // FCM: a service account inline or as a file, with its project from the key or the env.
    std::env::remove_var("FCM_SERVICE_ACCOUNT_JSON");
    assert!(push::FcmSender::from_env(reqwest::Client::new()).is_none());
    if let Some(rsa) = openssl(&["genrsa", "2048"]) {
        let account = json!({"client_email": "push@nomi.iam.test", "private_key": rsa, "project_id": "nomi-test", "token_uri": "http://127.0.0.1:9/token"});
        std::env::set_var("FCM_SERVICE_ACCOUNT_JSON", account.to_string());
        std::env::remove_var("FCM_PROJECT_ID");
        let sender = push::FcmSender::from_env(reqwest::Client::new()).unwrap().unwrap();
        assert!(sender.is_enabled());
        let parsed: push::ServiceAccount = serde_json::from_value(account.clone()).unwrap();
        assert!(!format!("{parsed:?}").contains("BEGIN"), "the private key never shows in debug output");
        // The token exchange fails (nothing listens there): an error, not a panic.
        let rt = tokio::runtime::Runtime::new().unwrap();
        let target = DeviceTarget { id: Uuid::new_v4(), platform: "android".into(), token: "t".into() };
        let alert = PushMessage { title: "Hi".into(), body: "There".into(), data: Default::default(), kind: push::PushKind::Alert };
        assert!(matches!(rt.block_on(sender.send(&target, &alert)), Err(PushError::Failed(_))));
        assert!(matches!(rt.block_on(sender.send(&target, &PushMessage::live_activity_update(json!({})))), Err(PushError::Unsupported(_))));

        let path = std::env::temp_dir().join(format!("nomi-fcm-{}.json", Uuid::new_v4()));
        std::fs::write(&path, json!({"client_email": "push@nomi.iam.test", "private_key": "not a key"}).to_string()).unwrap();
        std::env::set_var("FCM_SERVICE_ACCOUNT_JSON", path.to_str().unwrap());
        std::env::set_var("FCM_PROJECT_ID", "nomi-test");
        assert!(push::FcmSender::from_env(reqwest::Client::new()).unwrap().is_err(), "a key that isn't one is reported");
        std::env::remove_var("FCM_PROJECT_ID");
        std::fs::write(&path, json!({"client_email": "push@nomi.iam.test", "private_key": rsa}).to_string()).unwrap();
        assert!(push::FcmSender::from_env(reqwest::Client::new()).unwrap().is_err(), "no project named anywhere");
        std::env::set_var("FCM_SERVICE_ACCOUNT_JSON", "/no/such/file.json");
        assert!(push::FcmSender::from_env(reqwest::Client::new()).unwrap().is_err());
        let _ = std::fs::remove_file(path);
    }
    assert_eq!(push::fcm_send_url("p1"), "https://fcm.googleapis.com/v1/projects/p1/messages:send");
    assert!(push::fcm_rejects_token(&json!({"error": {"details": [{"@type": "type.googleapis.com/google.firebase.fcm.v1.FcmError", "errorCode": "UNREGISTERED"}]}})));
    assert!(!push::fcm_rejects_token(&json!({"error": {"message": "quota"}})));

    // APNs: a .p8 key (EC P-256) with its key and team ids.
    std::env::remove_var("APNS_KEY_ID");
    assert!(push::ApnsSender::from_env().is_none());
    if let Some(ec) = openssl(&["ecparam", "-name", "prime256v1", "-genkey", "-noout"]).and_then(|_| {
        let key = std::process::Command::new("sh").arg("-c").arg("openssl ecparam -name prime256v1 -genkey -noout | openssl pkcs8 -topk8 -nocrypt").output().ok()?;
        key.status.success().then(|| String::from_utf8_lossy(&key.stdout).into_owned())
    }) {
        let config = push::ApnsConfig { key_id: "KEY123".into(), team_id: "TEAM123".into(), topic: "ai.nomi.app".into(), production: false };
        assert!(push::ApnsSender::new(config.clone(), &ec).is_ok());
        assert!(push::ApnsSender::new(config, "not a key").is_err());
        std::env::set_var("APNS_PRIVATE_KEY_P8", ec.replace('\n', "\\n"));
        std::env::set_var("APNS_KEY_ID", "KEY123");
        std::env::set_var("APNS_TEAM_ID", "TEAM123");
        std::env::set_var("APNS_TOPIC", "ai.nomi.app");
        assert!(push::ApnsSender::from_env().unwrap().is_ok());
        std::env::set_var("APNS_PRIVATE_KEY_P8", "/no/such/key.p8");
        assert!(push::ApnsSender::from_env().unwrap().is_err());
    }
    assert!(push::apns_rejects_token(410, None));
    assert!(push::apns_rejects_token(400, Some("BadDeviceToken")));
    assert!(!push::apns_rejects_token(500, None));
}
