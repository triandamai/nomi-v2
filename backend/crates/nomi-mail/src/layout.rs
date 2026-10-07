//! The one layout every email to a person goes through, so each reads as Nomi: the sending
//! agent's shape (with its face) and colours up top, Nomi's type, and the crew along the bottom.
//! Table-based with inline styles, since that's what mail apps render reliably; the shapes travel
//! inside the email as PNGs (see `brand`).

use crate::brand::{self, NOMI_LOOK};
use crate::{Email, InlineImage};

/// Who an email is from: Nomi, or one crew member wearing its own look.
#[derive(Debug, Clone, PartialEq)]
pub struct Sender {
    pub name: String,
    pub shape: String,
    pub tone: String,
    is_nomi: bool,
}

impl Sender {
    pub fn nomi() -> Self {
        Self { name: "Nomi".into(), shape: NOMI_LOOK.shape.into(), tone: NOMI_LOOK.tone.into(), is_nomi: true }
    }

    /// A built-in crew member, by agent_type ("money") and the name people see ("Money").
    pub fn crew(agent: &str, display_name: &str) -> Self {
        let look = brand::agent_look(agent);
        if look == NOMI_LOOK {
            return Self::nomi();
        }
        Self { name: display_name.into(), shape: look.shape.into(), tone: look.tone.into(), is_nomi: false }
    }

    /// A custom agent wearing the shape and tone it was given (unknown ones fall back to Nomi's).
    pub fn custom(name: &str, shape: &str, tone: &str) -> Self {
        let shape = if brand::is_shape(shape) { shape } else { NOMI_LOOK.shape };
        let tone = if brand::is_tone(tone) { tone } else { NOMI_LOOK.tone };
        Self { name: name.into(), shape: shape.into(), tone: tone.into(), is_nomi: false }
    }
}

/// An email's content; `build` lays it out.
#[derive(Debug, Clone)]
pub struct BrandedEmail {
    pub sender: Sender,
    pub to: String,
    pub subject: String,
    /// The line inbox lists show after the subject.
    pub preheader: String,
    pub title: String,
    pub paragraphs: Vec<String>,
    /// Something to copy, shown large in the sender's colour (a sign-in code).
    pub highlight: Option<String>,
    /// Smaller print under the main text.
    pub notes: Vec<String>,
    /// The tag under a crew member's name, e.g. "Nomi crew".
    pub crew_label: String,
    /// The last line, e.g. why they got this email.
    pub footer: String,
}

const FONT_BRAND: &str = "'Bricolage Grotesque','Trebuchet MS',Helvetica,Arial,sans-serif";
const FONT_PLAIN: &str = "'Instrument Sans',-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif";
const FONT_MONO: &str = "'JetBrains Mono',ui-monospace,SFMono-Regular,Menlo,Consolas,monospace";
const FONTS_LINK: &str = "https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:wght@700;800&family=Instrument+Sans:wght@400;600&family=JetBrains+Mono:wght@600&display=swap";

const PAGE: &str = "#f4f7f2";
const CARD: &str = "#ffffff";
const INK: &str = "#0f1a14";
const INK_SOFT: &str = "#47544b";
const LINE: &str = "#dfe7e0";

/// The crew along the bottom of every email.
const CREW: &[&str] = &["nomi", "money", "reminders", "planning", "coding", "personality"];

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn image(content_id: &str, shape: &str, tone: &str, px: u32) -> InlineImage {
    InlineImage { content_id: content_id.into(), png: brand::shape_png(shape, tone, true, px) }
}

impl BrandedEmail {
    pub fn build(self) -> Email {
        let tone = brand::tone(&self.sender.tone);
        let band = if tone.stops.len() > 1 {
            format!("background-color:{};background-image:linear-gradient(90deg,{});", tone.stops[0], tone.stops.join(","))
        } else {
            format!("background-color:{};", tone.stops[0])
        };
        let tint = brand::lighten(tone.stops[0], 0.55);

        let mut images = vec![image("sender", &self.sender.shape, tone.name, 144)];
        let crew_cells: String = CREW
            .iter()
            .map(|agent| {
                let id = format!("crew-{agent}");
                let look = brand::agent_look(agent);
                images.push(image(&id, look.shape, look.tone, 40));
                format!(r#"<td style="padding:0 3px"><img src="cid:{id}" width="20" height="20" alt="" style="display:block;border:0"></td>"#)
            })
            .collect();

        let name = escape(&self.sender.name);
        let identity = if self.sender.is_nomi {
            format!(r#"<td style="padding-left:12px;font-family:{FONT_BRAND};font-size:28px;line-height:1;font-weight:800;letter-spacing:-1.2px;color:{INK}" class="ink">nomi</td>"#)
        } else {
            format!(
                r#"<td style="padding-left:12px"><div style="font-family:{FONT_BRAND};font-size:22px;line-height:1.1;font-weight:800;letter-spacing:-0.6px;color:{INK}" class="ink">{name}</div><div style="padding-top:4px;font-family:{FONT_MONO};font-size:11px;letter-spacing:1px;text-transform:uppercase;color:{accent}" class="accent">{label}</div></td>"#,
                accent = tone.accent,
                label = escape(&self.crew_label),
            )
        };

        let paragraphs: String = self
            .paragraphs
            .iter()
            .map(|p| format!(r#"<p style="margin:0 0 14px;font-family:{FONT_PLAIN};font-size:16px;line-height:1.55;color:{INK}" class="ink">{}</p>"#, escape(p)))
            .collect();
        let highlight = self
            .highlight
            .as_deref()
            .map(|h| {
                format!(
                    r#"<tr><td style="padding:6px 32px 18px"><div style="background-color:{tint};border-radius:20px;padding:18px 12px;text-align:center;font-family:{FONT_MONO};font-size:34px;line-height:1.1;font-weight:600;letter-spacing:10px;color:{accent}" class="highlight">{}</div></td></tr>"#,
                    escape(h),
                    accent = tone.accent,
                )
            })
            .unwrap_or_default();
        let notes: String = self
            .notes
            .iter()
            .map(|n| format!(r#"<p style="margin:0 0 10px;font-family:{FONT_PLAIN};font-size:14px;line-height:1.5;color:{INK_SOFT}" class="soft">{}</p>"#, escape(n)))
            .collect();

        let html = format!(
            r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="color-scheme" content="light dark"><meta name="supported-color-schemes" content="light dark">
<title>{subject}</title>
<link href="{FONTS_LINK}" rel="stylesheet">
<style>
@media (prefers-color-scheme: dark) {{
  .page {{ background-color:#0d1511 !important; }}
  .card {{ background-color:#14201a !important; }}
  .ink {{ color:#e3ede5 !important; }}
  .soft {{ color:#b4c2b8 !important; }}
  .rule {{ border-color:#2a3a31 !important; }}
  .highlight {{ background-color:#1d2c24 !important; color:{dark_accent} !important; }}
  .accent {{ color:{dark_accent} !important; }}
}}
</style></head>
<body style="margin:0;padding:0;background-color:{PAGE}" class="page">
<div style="display:none;max-height:0;overflow:hidden;opacity:0;color:transparent">{preheader}</div>
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" style="background-color:{PAGE}" class="page"><tr><td align="center" style="padding:32px 16px">
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" style="max-width:480px;background-color:{CARD};border-radius:28px;overflow:hidden" class="card">
<tr><td style="height:8px;line-height:8px;font-size:0;{band}">&nbsp;</td></tr>
<tr><td style="padding:28px 32px 0"><table role="presentation" cellpadding="0" cellspacing="0" border="0"><tr>
<td style="width:48px"><img src="cid:sender" width="48" height="48" alt="{name}" style="display:block;border:0"></td>{identity}
</tr></table></td></tr>
<tr><td style="padding:26px 32px 4px"><h1 style="margin:0 0 14px;font-family:{FONT_BRAND};font-size:26px;line-height:1.2;font-weight:800;letter-spacing:-0.5px;color:{INK}" class="ink">{title}</h1>{paragraphs}</td></tr>
{highlight}
<tr><td style="padding:0 32px 8px">{notes}<p style="margin:8px 0 0;font-family:{FONT_PLAIN};font-size:15px;line-height:1.5;font-weight:600;color:{INK}" class="ink">— {name}</p></td></tr>
<tr><td style="padding:22px 32px 26px"><div style="border-top:1px solid {LINE};padding-top:18px" class="rule">
<table role="presentation" cellpadding="0" cellspacing="0" border="0"><tr>{crew_cells}</tr></table>
<p style="margin:10px 0 0;font-family:{FONT_PLAIN};font-size:12px;line-height:1.5;color:{INK_SOFT}" class="soft">{footer}</p>
</div></td></tr>
</table>
</td></tr></table>
</body></html>"#,
            subject = escape(&self.subject),
            preheader = escape(&self.preheader),
            title = escape(&self.title),
            footer = escape(&self.footer),
            dark_accent = brand::lighten(tone.accent, 0.45),
        );

        let mut text = format!("{}\n\n", self.title);
        for p in &self.paragraphs {
            text.push_str(&format!("{p}\n\n"));
        }
        if let Some(h) = &self.highlight {
            text.push_str(&format!("    {h}\n\n"));
        }
        for n in &self.notes {
            text.push_str(&format!("{n}\n\n"));
        }
        text.push_str(&format!("— {}\n\n{}\n", self.sender.name, self.footer));

        Email { to: self.to, subject: self.subject, text, html: Some(html), inline_images: images }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn email(sender: Sender) -> Email {
        BrandedEmail {
            sender,
            to: "ana@example.com".into(),
            subject: "Your code: 123456".into(),
            preheader: "It works for 10 minutes.".into(),
            title: "Here's your code".into(),
            paragraphs: vec!["Enter it to sign in <now>.".into()],
            highlight: Some("123456".into()),
            notes: vec!["Didn't ask? Ignore this.".into()],
            crew_label: "Nomi crew".into(),
            footer: "You got this because someone signed in.".into(),
        }
        .build()
    }

    #[test]
    fn nomi_signs_with_her_shape_and_wordmark() {
        let e = email(Sender::nomi());
        let html = e.html.unwrap();
        assert!(html.contains(r#"src="cid:sender""#));
        assert!(html.contains(">nomi</td>"));
        assert!(html.contains("#e4f76a"), "Nomi's glow gradient");
        assert!(html.contains("123456"));
        assert!(html.contains("&lt;now&gt;"), "text is escaped");
        assert_eq!(e.inline_images.len(), 1 + CREW.len());
        assert!(e.inline_images.iter().all(|i| &i.png[1..4] == b"PNG"));
        assert!(e.text.contains("    123456") && e.text.contains("— Nomi"));
    }

    #[test]
    fn a_crew_member_signs_in_its_own_colours() {
        let html = email(Sender::crew("money", "Money")).html.unwrap();
        assert!(html.contains("#ffd27a") && html.contains("#b8430f"), "ember gradient and accent");
        assert!(html.contains(">Money</div>") && html.contains("Nomi crew"));
        assert!(html.contains("— Money"));
    }

    #[test]
    fn a_custom_agent_wears_its_pick_or_nomis() {
        assert_eq!(Sender::custom("Chef", "pill", "bloom").shape, "pill");
        let fallback = Sender::custom("Chef", "hexagon", "neon");
        assert_eq!((fallback.shape.as_str(), fallback.tone.as_str()), ("cookie9", "glow"));
        assert_eq!(Sender::crew("chitchat", "Nomi"), Sender::nomi());
    }
}
