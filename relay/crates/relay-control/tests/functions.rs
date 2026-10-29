//! A project's functions per environment: each deploy is a numbered build, checked by the
//! functions runtime (wiremock stands in for it) before it replaces the running one, and the
//! previous good build comes back on rollback.
use std::collections::HashMap;

use relay_auth::config::set_secret;
use relay_control::backend::ensure_backend;
use relay_control::functions::{self, SourceFile};
use relay_control::{Relay, RelayError};
use relay_core::backend::{Access, FunctionSettings};
use relay_core::settings::FunctionsRuntime;
use relay_core::{Env, Tier};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::collections::BTreeMap;
use uuid::Uuid;
use wiremock::matchers::{header, method, path_regex};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const SECRET: &str = "runtime-secret-runtime-secret-runtime";

async fn world(pool: &PgPool) -> (Relay, Uuid, MockServer) {
    let runtime = MockServer::start().await;
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let uri = runtime.uri();
    let relay = Relay::new(pool.clone(), shared, [9u8; 32], "api.relay.test".to_string())
        .with_settings(|s| s.functions = Some(FunctionsRuntime { url: uri, secret: SECRET.into() }));
    let project = Uuid::new_v4();
    ensure_backend(&relay, project, "Shop", Tier::Free, None).await.unwrap();
    (relay, project, runtime)
}

fn files(body: &str) -> Vec<SourceFile> {
    vec![
        SourceFile { path: "nomi/functions/place-order.ts".into(), content: body.into() },
        SourceFile { path: "nomi/functions/_shared/money.ts".into(), content: "export const cents = (n: number) => n * 100;".into() },
    ]
}

fn loads_ok() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "ok": true, "functions": ["place-order"] }))
}

fn sent(request: &Request) -> Value {
    serde_json::from_slice(&request.body).unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_deploy_is_checked_by_the_runtime_and_becomes_the_current_build(pool: PgPool) {
    let (relay, project, runtime) = world(&pool).await;
    set_secret(&relay.state, &relay.encryption_key, project, "STRIPE_SECRET", Some("sk_test")).await.unwrap();
    Mock::given(method("PUT")).and(path_regex(format!("^/builds/{project}/dev/1$"))).and(header("x-relay-runtime", SECRET)).respond_with(loads_ok()).expect(1).mount(&runtime).await;
    let settings = BTreeMap::from([("place-order".to_string(), FunctionSettings { auth: Access::Public, timeout: 10, schedule: Some("0 * * * *".into()) })]);

    let deployed = functions::deploy(&relay, project, Env::Dev, &files("export default () => new Response('ok')"), &settings).await.unwrap();

    assert_eq!(deployed.version, 1);
    assert_eq!(deployed.functions["place-order"].timeout, 10);
    let request = &runtime.received_requests().await.unwrap()[0];
    let body = sent(request);
    assert_eq!(body["files"].as_array().unwrap().len(), 2);
    assert_eq!(body["env"]["STRIPE_SECRET"], "sk_test", "the project's secrets are ctx.env");
    assert!(body["env"]["NOMI_URL"].as_str().unwrap().ends_with("-dev.api.relay.test"));
    assert!(body["env"]["NOMI_ANON_KEY"].is_string() && body["env"]["NOMI_SERVICE_KEY"].is_string());
    let current = functions::current(&relay, project, Env::Dev).await.unwrap().unwrap();
    assert_eq!(current.version, 1);
    assert_eq!(current.functions["place-order"].auth, Access::Public);
    assert!(functions::current(&relay, project, Env::Production).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_build_that_fails_its_check_never_replaces_the_running_one(pool: PgPool) {
    let (relay, project, runtime) = world(&pool).await;
    Mock::given(method("PUT")).and(path_regex("/1$")).respond_with(loads_ok()).mount(&runtime).await;
    Mock::given(method("PUT"))
        .and(path_regex("/2$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ok": false, "error": "nomi/functions/place-order.ts:1:7 - error TS2322" })))
        .mount(&runtime)
        .await;
    functions::deploy(&relay, project, Env::Dev, &files("export default () => new Response('ok')"), &BTreeMap::new()).await.unwrap();

    let refused = functions::deploy(&relay, project, Env::Dev, &files("const n: number = 'x';"), &BTreeMap::new()).await.unwrap_err();

    let RelayError::Rejected { code, message, .. } = refused else { panic!("not a refusal") };
    assert_eq!(code, "FUNCTIONS_FAILED");
    assert!(message.contains("place-order.ts:1:7"), "{message}");
    assert_eq!(functions::current(&relay, project, Env::Dev).await.unwrap().unwrap().version, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn rollback_brings_back_the_previous_good_build(pool: PgPool) {
    let (relay, project, runtime) = world(&pool).await;
    Mock::given(method("PUT")).respond_with(loads_ok()).mount(&runtime).await;
    functions::deploy(&relay, project, Env::Production, &files("export default () => new Response('one')"), &BTreeMap::new()).await.unwrap();
    functions::deploy(&relay, project, Env::Production, &files("export default () => new Response('two')"), &BTreeMap::new()).await.unwrap();

    let back = functions::rollback(&relay, project, Env::Production).await.unwrap();

    assert_eq!(back.version, 1);
    assert_eq!(functions::current(&relay, project, Env::Production).await.unwrap().unwrap().version, 1);
    let reloaded = runtime.received_requests().await.unwrap().last().map(|r| r.url.path().to_string()).unwrap();
    assert!(reloaded.ends_with("/production/1"), "{reloaded}");
    assert!(functions::rollback(&relay, project, Env::Production).await.is_err(), "nothing older to go back to");
}

#[sqlx::test(migrations = "../../migrations")]
async fn settings_for_a_function_with_no_file_and_bad_settings_are_refused(pool: PgPool) {
    let (relay, project, _runtime) = world(&pool).await;
    let cases = [
        BTreeMap::from([("refund".to_string(), FunctionSettings::default())]),
        BTreeMap::from([("place-order".to_string(), FunctionSettings { timeout: 61, ..FunctionSettings::default() })]),
        BTreeMap::from([("place-order".to_string(), FunctionSettings { schedule: Some("every hour".into()), ..FunctionSettings::default() })]),
    ];
    for settings in cases {
        let refused = functions::deploy(&relay, project, Env::Dev, &files("export default () => new Response()"), &settings).await.unwrap_err();
        assert!(matches!(refused, RelayError::Rejected { code: "BACKEND_INVALID", .. }), "{refused:?}");
    }
    let none = functions::deploy(&relay, project, Env::Dev, &[], &BTreeMap::new()).await.unwrap_err();
    assert!(matches!(none, RelayError::Rejected { code: "BACKEND_INVALID", .. }), "{none:?}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn without_a_runtime_set_functions_are_refused(pool: PgPool) {
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let relay = Relay::new(pool.clone(), shared, [9u8; 32], "api.relay.test".to_string());
    let project = Uuid::new_v4();
    ensure_backend(&relay, project, "Shop", Tier::Free, None).await.unwrap();

    let refused = functions::deploy(&relay, project, Env::Dev, &files("export default () => new Response()"), &BTreeMap::new()).await.unwrap_err();

    assert!(matches!(refused, RelayError::Rejected { code: "FUNCTIONS_NOT_SET_UP", .. }), "{refused:?}");
}
