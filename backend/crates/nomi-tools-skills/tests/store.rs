use nomi_skills::{validate, ParsedSkill, Reference, MAX_SKILLS_PER_MEMBER};
use nomi_tools_skills::store::{self, Origin, StoreError};
use sqlx::PgPool;
use uuid::Uuid;

const CHEF: &str = "chef";

async fn user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

fn skill(name: &str) -> ParsedSkill {
    let text = format!("---\nname: {name}\ndescription: Use when planning meals for the week.\n---\n# {name}\nBody");
    validate(&text, vec![Reference { name: "pantry".into(), text: "rice, eggs".into() }]).unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn insert_then_list_and_get(pool: PgPool) {
    let me = user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let row = store::insert(&mut conn, me, CHEF, &skill("menu"), Origin::Typed, None).await.unwrap();
    assert_eq!(row.name, "menu");
    assert_eq!(row.description, "Use when planning meals for the week.");
    assert_eq!(row.reference_names, vec!["pantry".to_string()]);
    assert!(row.enabled);
    assert_eq!(row.origin, Origin::Typed);

    let listed = store::list(&mut conn, me, CHEF).await.unwrap();
    assert_eq!(listed.len(), 1);
    let (got, references) = store::get(&mut conn, me, CHEF, row.id).await.unwrap().unwrap();
    assert!(got.text.contains("# menu"));
    assert_eq!(references, vec![Reference { name: "pantry".into(), text: "rice, eggs".into() }]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn name_taken_for_same_member_only(pool: PgPool) {
    let me = user(&pool).await;
    let other = user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    store::insert(&mut conn, me, CHEF, &skill("menu"), Origin::Typed, None).await.unwrap();
    assert!(matches!(store::insert(&mut conn, me, CHEF, &skill("menu"), Origin::Upload, None).await, Err(StoreError::NameTaken)));
    store::insert(&mut conn, me, "sora", &skill("menu"), Origin::Typed, None).await.unwrap();
    store::insert(&mut conn, other, CHEF, &skill("menu"), Origin::Typed, None).await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn thirty_first_skill_is_too_many(pool: PgPool) {
    let me = user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    for i in 0..MAX_SKILLS_PER_MEMBER {
        let row = store::insert(&mut conn, me, CHEF, &skill(&format!("s{i}")), Origin::Typed, None).await.unwrap();
        if i == 0 {
            assert!(store::set_enabled(&mut conn, me, CHEF, row.id, false).await.unwrap(), "a switched-off one still counts");
        }
    }
    assert!(matches!(store::insert(&mut conn, me, CHEF, &skill("one-more"), Origin::Typed, None).await, Err(StoreError::TooMany)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn parallel_inserts_at_the_cap_land_one(pool: PgPool) {
    let me = user(&pool).await;
    {
        let mut conn = pool.acquire().await.unwrap();
        for i in 0..MAX_SKILLS_PER_MEMBER - 1 {
            store::insert(&mut conn, me, CHEF, &skill(&format!("s{i}")), Origin::Typed, None).await.unwrap();
        }
    }
    let (a, b) = (pool.clone(), pool.clone());
    let one = tokio::spawn(async move { store::insert(&mut a.acquire().await.unwrap(), me, CHEF, &skill("left"), Origin::Typed, None).await });
    let two = tokio::spawn(async move { store::insert(&mut b.acquire().await.unwrap(), me, CHEF, &skill("right"), Origin::Typed, None).await });
    let results = [one.await.unwrap(), two.await.unwrap()];
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|r| matches!(r, Err(StoreError::TooMany))).count(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn another_users_skills_are_invisible(pool: PgPool) {
    let me = user(&pool).await;
    let other = user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let row = store::insert(&mut conn, me, CHEF, &skill("menu"), Origin::Typed, None).await.unwrap();
    assert!(store::list(&mut conn, other, CHEF).await.unwrap().is_empty());
    assert!(store::get(&mut conn, other, CHEF, row.id).await.unwrap().is_none());
    assert!(store::find_enabled(&mut conn, other, CHEF, "menu").await.unwrap().is_none());
    assert!(store::index(&mut conn, other, CHEF).await.unwrap().is_empty());
    assert!(!store::set_enabled(&mut conn, other, CHEF, row.id, false).await.unwrap());
    assert!(!store::delete(&mut conn, other, CHEF, row.id).await.unwrap());
    assert!(matches!(store::replace(&mut conn, other, CHEF, row.id, &skill("menu"), None, None).await, Err(StoreError::NotFound)));
    assert!(store::get(&mut conn, me, "sora", row.id).await.unwrap().is_none(), "another member's id isn't this member's");
    assert_eq!(store::list(&mut conn, me, CHEF).await.unwrap().len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn index_skips_disabled(pool: PgPool) {
    let me = user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let off = store::insert(&mut conn, me, CHEF, &skill("zeta"), Origin::Typed, None).await.unwrap();
    store::insert(&mut conn, me, CHEF, &skill("alpha"), Origin::Typed, None).await.unwrap();
    store::set_enabled(&mut conn, me, CHEF, off.id, false).await.unwrap();
    let index = store::index(&mut conn, me, CHEF).await.unwrap();
    assert_eq!(index.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), vec!["alpha"]);
    assert!(store::find_enabled(&mut conn, me, CHEF, "zeta").await.unwrap().is_none());
    assert!(store::find_enabled(&mut conn, me, CHEF, "alpha").await.unwrap().is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn replace_renames_when_free_else_name_taken(pool: PgPool) {
    let me = user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let menu = store::insert(&mut conn, me, CHEF, &skill("menu"), Origin::Typed, None).await.unwrap();
    store::insert(&mut conn, me, CHEF, &skill("pantry"), Origin::Typed, None).await.unwrap();
    let renamed = store::replace(&mut conn, me, CHEF, menu.id, &skill("weekly-menu"), None, None).await.unwrap();
    assert_eq!(renamed.name, "weekly-menu");
    assert_eq!(renamed.id, menu.id);
    assert!(matches!(store::replace(&mut conn, me, CHEF, menu.id, &skill("pantry"), None, None).await, Err(StoreError::NameTaken)));
    let bare = validate("---\nname: weekly-menu\ndescription: Use when planning meals for the week.\n---\nnew", vec![]).unwrap();
    let replaced = store::replace(&mut conn, me, CHEF, menu.id, &bare, None, None).await.unwrap();
    assert!(replaced.reference_names.is_empty(), "references are replaced too");
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_and_delete_member(pool: PgPool) {
    let me = user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let one = store::insert(&mut conn, me, CHEF, &skill("one"), Origin::Typed, None).await.unwrap();
    store::insert(&mut conn, me, CHEF, &skill("two"), Origin::Typed, None).await.unwrap();
    store::insert(&mut conn, me, "sora", &skill("three"), Origin::Typed, None).await.unwrap();
    assert!(store::delete(&mut conn, me, CHEF, one.id).await.unwrap());
    assert_eq!(store::delete_member(&mut conn, me, CHEF).await.unwrap(), 1);
    assert!(store::list(&mut conn, me, CHEF).await.unwrap().is_empty());
    assert_eq!(store::list(&mut conn, me, "sora").await.unwrap().len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn link_source_defaults_to_url(pool: PgPool) {
    let me = user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let url = "https://github.com/acme/skills/tree/main/menu";
    let row = store::insert(&mut conn, me, CHEF, &skill("menu"), Origin::Link, Some(url)).await.unwrap();
    assert_eq!(row.source, url);
    assert_eq!(row.source_url.as_deref(), Some(url));
    let mut with_source = skill("other");
    with_source.meta.source = "https://example.com/own".into();
    let row = store::insert(&mut conn, me, CHEF, &with_source, Origin::Link, Some(url)).await.unwrap();
    assert_eq!(row.source, "https://example.com/own", "a written source is kept");
}
