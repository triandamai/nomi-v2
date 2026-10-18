use nomi_realtime::UserEvent;
use nomi_server::user_events;

#[test]
fn preview_cuts_on_char_boundary() {
    let long = "é".repeat(500);
    let p = user_events::preview(&long);
    assert_eq!(p.chars().count(), user_events::CHAT_REPLY_PREVIEW_CHARS);
}

#[test]
fn preview_keeps_short_text() {
    assert_eq!(user_events::preview("hi there"), "hi there");
}

#[test]
fn notification_event_wire_shape() {
    let id = uuid::Uuid::nil();
    let v = serde_json::to_value(UserEvent::NotificationCreated { notification_id: id, kind: "quota".into(), title: "T".into(), body: "B".into(), link: None }).unwrap();
    assert_eq!(v["kind"], "NotificationCreated");
    assert_eq!(v["title"], "T");
}
