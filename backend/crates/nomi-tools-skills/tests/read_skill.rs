use nomi_skills::{validate, BuiltinSkill, Reference, SkillEntry};
use nomi_tool::{OfferContext, Tool, ToolContext};
use nomi_tools_skills::store::{self, Origin};
use nomi_tools_skills::{merged_index, tools, ReadSkillCopy};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

const CHEF: &str = "chef";
const COPY: ReadSkillCopy = ReadSkillCopy { description: "Read one of your skills.\n\n{index}" };

const BUILTIN: &[BuiltinSkill] = &[
    BuiltinSkill { name: "menu", text: "---\nname: menu\ndescription: The built-in menu planning guide.\n---\n# Built-in menu", references: &[] },
    BuiltinSkill {
        name: "plating",
        text: "---\nname: plating\ndescription: How to plate a dish so it looks good.\n---\n# Plating",
        references: &[("colours", "Use contrast.")],
    },
];

async fn user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

async fn add(pool: &PgPool, user_id: Uuid, name: &str, references: Vec<Reference>) -> Uuid {
    let text = format!("---\nname: {name}\ndescription: My own {name} skill, for this week.\n---\n# My {name}\nBody");
    let mut conn = pool.acquire().await.unwrap();
    store::insert(&mut conn, user_id, CHEF, &validate(&text, references).unwrap(), Origin::Typed, None).await.unwrap().id
}

fn tool() -> Box<dyn Tool> {
    tools(COPY).pop().unwrap()
}

async fn read(pool: &PgPool, user_id: Uuid, input: serde_json::Value) -> Result<String, String> {
    let mut conn = pool.acquire().await.unwrap();
    let mut ctx = ToolContext::new(Uuid::new_v4(), Uuid::new_v4(), user_id, CHEF, "Chef");
    ctx.builtin_skills = BUILTIN;
    tool().execute(&mut conn, &ctx, input).await.map(|o| o.display_text)
}

#[test]
fn properties_and_not_offered_without_skills() {
    let t = tool();
    assert_eq!(t.definition().name, "read_skill");
    assert_eq!(t.approval(), nomi_tool::Approval::Never);
    assert_eq!(t.auto_offer(), nomi_tool::AutoOffer::Everyone);
    assert!(t.posts_activity());
    assert!(t.offer(&OfferContext { reminder_targets: &[], mentions_files: false, skills: &[] }).is_none());
}

#[test]
fn offer_lists_mine_before_builtin() {
    let mine = vec![SkillEntry { name: "menu".into(), description: "My own menu skill, for this week.".into() }];
    let merged = merged_index(BUILTIN, mine);
    assert_eq!(merged.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), vec!["menu", "plating"], "mine first; the shadowed built-in is gone");
    let offered = tool().offer(&OfferContext { reminder_targets: &[], mentions_files: false, skills: &merged }).unwrap();
    assert!(offered.description.starts_with("Read one of your skills."));
    assert!(offered.description.contains("- menu: My own menu skill, for this week.\n- plating: How to plate a dish so it looks good."));
    assert!(!offered.description.contains("{index}"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn reads_body_and_names_references(pool: PgPool) {
    let me = user(&pool).await;
    add(&pool, me, "pantry", vec![Reference { name: "staples".into(), text: "rice".into() }, Reference { name: "spices".into(), text: "cumin".into() }]).await;
    let text = read(&pool, me, json!({"name": "pantry"})).await.unwrap();
    assert!(text.starts_with("# My pantry\nBody"));
    assert!(!text.contains("description:"), "no frontmatter");
    assert!(text.contains("spices, staples"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn reads_one_reference_case_insensitively(pool: PgPool) {
    let me = user(&pool).await;
    add(&pool, me, "pantry", vec![Reference { name: "staples".into(), text: "rice".into() }]).await;
    assert_eq!(read(&pool, me, json!({"name": "Pantry", "reference": "STAPLES.md"})).await.unwrap(), "rice");
    assert_eq!(read(&pool, me, json!({"name": "plating", "reference": "Colours"})).await.unwrap(), "Use contrast.", "a built-in one's too");
    let missing = read(&pool, me, json!({"name": "pantry", "reference": "sauces"})).await.unwrap_err();
    assert!(missing.contains("staples"), "{missing}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn mine_shadows_builtin_of_same_name(pool: PgPool) {
    let me = user(&pool).await;
    assert!(read(&pool, me, json!({"name": "menu"})).await.unwrap().contains("# Built-in menu"));
    add(&pool, me, "menu", vec![]).await;
    assert!(read(&pool, me, json!({"name": "menu"})).await.unwrap().contains("# My menu"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn unknown_name_lists_available(pool: PgPool) {
    let me = user(&pool).await;
    add(&pool, me, "pantry", vec![]).await;
    let err = read(&pool, me, json!({"name": "sauces"})).await.unwrap_err();
    assert!(err.contains("pantry") && err.contains("menu") && err.contains("plating"), "{err}");
    assert!(read(&pool, me, json!({})).await.is_err(), "name is required");
}

#[sqlx::test(migrations = "../../migrations")]
async fn disabled_skill_is_not_read(pool: PgPool) {
    let me = user(&pool).await;
    let id = add(&pool, me, "pantry", vec![]).await;
    store::set_enabled(&mut pool.acquire().await.unwrap(), me, CHEF, id, false).await.unwrap();
    assert!(read(&pool, me, json!({"name": "pantry"})).await.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn other_users_skill_is_not_read(pool: PgPool) {
    let me = user(&pool).await;
    let other = user(&pool).await;
    add(&pool, other, "pantry", vec![]).await;
    let err = read(&pool, me, json!({"name": "pantry"})).await.unwrap_err();
    assert!(!err.contains("pantry,") && !err.contains("pantry\n"), "{err}");
}
