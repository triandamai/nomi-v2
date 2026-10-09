//! Writes sample emails as HTML files you can open in a browser, with the agents' shapes inlined
//! (mail apps get them as attachments instead):
//!     cargo run -p nomi-mail --example preview -- <out dir>

use nomi_mail::layout::{BrandedEmail, Sender};

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "email-previews".into());
    std::fs::create_dir_all(&out).expect("create the output folder");
    let samples = [
        ("nomi-code", Sender::nomi(), "Your sign-in code", vec!["Here's your code to sign in to Nomi:".to_string()], Some("482913")),
        (
            "money",
            Sender::crew("money", "Finley"),
            "You're close to your food budget",
            vec!["You've spent Rp1.840.000 of Rp2.000.000 on food this month, with 9 days to go.".into(), "Want me to suggest a few easy swaps for the rest of the month?".into()],
            None,
        ),
        (
            "reminders",
            Sender::crew("reminders", "Cadence"),
            "Book flights before the fare jumps",
            vec!["You asked me to remind you today at 13:52.".into()],
            None,
        ),
        ("custom", Sender::custom("Chef", "pill", "bloom"), "Tonight's dinner plan", vec!["Three dishes you can make with what's in your fridge.".into()], None),
    ];
    for (file, sender, title, paragraphs, highlight) in samples {
        let email = BrandedEmail {
            sender,
            to: "ana@example.com".into(),
            subject: title.into(),
            preheader: "A preview".into(),
            title: title.into(),
            paragraphs,
            highlight: highlight.map(String::from),
            notes: vec!["If this wasn't you, you can ignore this email.".into()],
            crew_label: "Nomi crew".into(),
            footer: "Nomi · a small crew of agents that remembers you.".into(),
        }
        .build();
        let mut html = email.html.unwrap_or_default();
        for image in &email.inline_images {
            let data = format!("data:image/png;base64,{}", base64(&image.png));
            html = html.replace(&format!("cid:{}", image.content_id), &data);
        }
        let path = format!("{out}/{file}.html");
        std::fs::write(&path, html).expect("write the preview");
        println!("{path}");
    }
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}
