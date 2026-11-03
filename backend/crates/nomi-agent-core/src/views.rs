//! Views: how the crew shows something in a chat bubble when plain text isn't the best fit — a
//! list, a timeline, a chart, a form to fill in. Any agent shows one with the `show` tool, and an
//! agent's own tools can return one built from data they just saved (e.g. the Money agent's
//! transaction card). Either way it is posted as a `ContentBlock::View`, and the frontend renders
//! it with the view's component (frontend/src/lib/components/views).
//!
//! To add a view:
//! 1. its data shape here (a struct, a `View` variant, and its `check` and `plain_text`),
//! 2. its line in `CATALOG` (what the model reads in the `show` tool's description),
//! 3. an example in frontend/src/lib/views/examples.json (both sides' tests parse every example),
//! 4. its component, type and registry entry in frontend/src/lib/views.
//!
//! The data is checked before it is posted: a view that doesn't fit its shape goes back to the
//! agent as an error to fix, never into the chat. Unknown extra fields are ignored rather than
//! refused, so a model adding one doesn't cost a retry.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const SHOW_TOOL_NAME: &str = "show";

/// Most items, rows, steps or events one view holds.
const MAX_ITEMS: usize = 60;
/// Most characters of a shell view's output (the end is kept: that is where results and errors are).
const MAX_SHELL_OUTPUT: usize = 8_000;
/// Most characters of a code view.
const MAX_CODE: usize = 20_000;
/// Most lines on each side of a diff (the line-by-line comparison is quadratic).
const MAX_DIFF_LINES: usize = 800;
/// Unchanged lines kept around each change in a diff.
const DIFF_CONTEXT: usize = 3;

/// The hunks that turn `before` into `after`, line by line (a longest-common-subsequence diff),
/// each with `context` unchanged lines around its changes, as `diff -u` writes them.
pub fn diff_hunks(before: &str, after: &str, context: usize) -> Vec<DiffHunk> {
    let a: Vec<&str> = before.lines().collect();
    let b: Vec<&str> = after.lines().collect();
    let (n, m) = (a.len(), b.len());
    // lcs[i][j]: the longest common run of a[i..] and b[j..].
    let mut lcs = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if a[i] == b[j] { lcs[i + 1][j + 1] + 1 } else { lcs[i + 1][j].max(lcs[i][j + 1]) };
        }
    }
    // Every line in order, with its numbers on each side.
    let mut all: Vec<DiffLine> = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && a[i] == b[j] {
            all.push(DiffLine { kind: DiffLineKind::Context, old: Some(i + 1), new: Some(j + 1), text: a[i].to_string() });
            i += 1;
            j += 1;
        } else if i < n && (j == m || lcs[i + 1][j] >= lcs[i][j + 1]) {
            // Removed lines come before the lines that replace them, as `diff -u` writes them.
            all.push(DiffLine { kind: DiffLineKind::Del, old: Some(i + 1), new: None, text: a[i].to_string() });
            i += 1;
        } else {
            all.push(DiffLine { kind: DiffLineKind::Add, old: None, new: Some(j + 1), text: b[j].to_string() });
            j += 1;
        }
    }
    // Group the changes, with their context, into hunks.
    let changed: Vec<usize> = all.iter().enumerate().filter(|(_, l)| l.kind != DiffLineKind::Context).map(|(k, _)| k).collect();
    let mut hunks = Vec::new();
    let mut k = 0;
    while k < changed.len() {
        let start = changed[k].saturating_sub(context);
        let mut end = changed[k];
        while k + 1 < changed.len() && changed[k + 1] <= end + 2 * context + 1 {
            k += 1;
            end = changed[k];
        }
        let end = (end + context + 1).min(all.len());
        let lines: Vec<DiffLine> = all[start..end].to_vec();
        let old_count = lines.iter().filter(|l| l.kind != DiffLineKind::Add).count();
        let new_count = lines.iter().filter(|l| l.kind != DiffLineKind::Del).count();
        // Where the hunk starts on each side: its first numbered line, or the line before it.
        let old_start = lines.iter().find_map(|l| l.old).unwrap_or_else(|| all[..start].iter().rev().find_map(|l| l.old).unwrap_or(0));
        let new_start = lines.iter().find_map(|l| l.new).unwrap_or_else(|| all[..start].iter().rev().find_map(|l| l.new).unwrap_or(0));
        hunks.push(DiffHunk { header: format!("@@ -{old_start},{old_count} +{new_start},{new_count} @@"), lines });
        k += 1;
    }
    hunks
}

/// A value as the person sees it: as written, or formatted for their language (an amount with
/// cents raised, a date in their style).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    #[default]
    Text,
    Number,
    /// An amount of money in whole units (e.g. 45000 or 12.5), shown in the person's currency style.
    Money,
    /// A fraction or a percentage number (0.4 or 40 both read as 40%).
    Percent,
    /// An ISO 8601 date.
    Date,
    /// An ISO 8601 date and time.
    DateTime,
}

impl Format {
    fn is_text(&self) -> bool {
        *self == Format::Text
    }
}

/// A single value: text, a number or yes/no.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Scalar {
    Bool(bool),
    Number(f64),
    Text(String),
}

impl std::fmt::Display for Scalar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scalar::Bool(b) => write!(f, "{}", if *b { "yes" } else { "no" }),
            Scalar::Number(n) if n.fract() == 0.0 && n.abs() < 1e15 => write!(f, "{}", *n as i64),
            Scalar::Number(n) => write!(f, "{n}"),
            Scalar::Text(s) => f.write_str(s),
        }
    }
}

/// How something stands, for views that colour it (a card, an answer).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    #[default]
    Neutral,
    Positive,
    Negative,
    Warning,
}

impl Tone {
    fn is_neutral(&self) -> bool {
        *self == Tone::Neutral
    }
}

/// A labelled value (a card's facts, an answer's figure).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub label: String,
    pub value: Scalar,
    #[serde(default, skip_serializing_if = "Format::is_text")]
    pub format: Format,
}

/// A link to open.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkRef {
    pub label: String,
    pub url: String,
}

/// A button that sends `message` as the person's next message (e.g. "Undo" → "Delete that transaction").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub label: String,
    pub message: String,
}

// ---- the views ----

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListStyle {
    #[default]
    Bullet,
    Numbered,
    Check,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// A short value at the end of the row (an amount, a time, a count).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Scalar>,
    /// How `meta` is shown.
    #[serde(default, skip_serializing_if = "Format::is_text")]
    pub format: Format,
    /// Ticked, for a `check` list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub style: ListStyle,
    pub items: Vec<ListItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Column {
    pub key: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Format::is_text")]
    pub format: Format,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub columns: Vec<Column>,
    /// One object per row, keyed by the columns' `key`.
    pub rows: Vec<Map<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparisonRow {
    pub label: String,
    /// One value per option, in the options' order.
    pub values: Vec<Scalar>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparisonView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// What is being compared (the columns).
    pub options: Vec<String>,
    /// The criteria (the rows).
    pub rows: Vec<ComparisonRow>,
    /// Index into `options` of the one to recommend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verdict: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Done,
    Current,
    #[default]
    Upcoming,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default)]
    pub status: StepStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepperView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineEvent {
    /// When: an ISO 8601 date or date-time (shown in the person's style), or a label like "Day 2".
    pub at: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub events: Vec<TimelineEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardView {
    /// A small line above the title (a category, a kind of thing).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eyebrow: Option<String>,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// The one figure that matters, shown large.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight: Option<Field>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<Field>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<LinkRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<Action>,
    #[serde(default, skip_serializing_if = "Tone::is_neutral")]
    pub tone: Tone,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartKind {
    #[default]
    Bar,
    Line,
    Donut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Series {
    pub name: String,
    /// One value per label.
    pub values: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub chart: ChartKind,
    /// The categories or points along the axis (a donut's slices).
    pub labels: Vec<String>,
    pub series: Vec<Series>,
    /// How the values are shown.
    #[serde(default, skip_serializing_if = "Format::is_text")]
    pub format: Format,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnswerView {
    /// What the value is ("Spent this month").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub value: Scalar,
    #[serde(default, skip_serializing_if = "Format::is_text")]
    pub format: Format,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Tone::is_neutral")]
    pub tone: Tone,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuggestionsView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Each is sent as the person's message when tapped, so write them in their voice.
    pub items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionOption {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pros: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cons: Vec<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub recommended: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionView {
    pub question: String,
    pub options: Vec<DecisionOption>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKind {
    #[default]
    Text,
    Textarea,
    Number,
    Date,
    Time,
    Select,
    Multiselect,
    Checkbox,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormField {
    pub key: String,
    pub label: String,
    #[serde(default, rename = "type")]
    pub kind: FieldKind,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub required: bool,
    /// The choices of a `select` or `multiselect`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Scalar>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intro: Option<String>,
    pub fields: Vec<FormField>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submit_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkView {
    pub url: String,
    /// Filled from the page itself when left out (see `crate::web::page_preview`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[default]
    Queued,
    Running,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskView {
    /// The background task this card follows (set by the server, so the card's status can be
    /// brought up to date when the task ends).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub status: TaskStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// How far along, 0 to 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    /// Which crew member it's with.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// The crew's own rough estimate of how long it takes ("10 min"), when it runs in the background.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eta: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShellView {
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alternative {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// The crew member who'd do it (an agent type like "money", or a crew name like "Dana").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Sent as the person's next message when they pick it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Or a page to open instead (a web address or an app page like /money).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Something the crew won't or can't do, the reason, and what it can do instead.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundaryView {
    pub title: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<Alternative>,
}

/// Short answers to the question just asked, one tap each.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuickRepliesView {
    pub replies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeView {
    pub code: String,
    /// The language, for the label and highlighting (typescript, rust, sql, bash…).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// The file it belongs in, when it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffLineKind {
    Context,
    Add,
    Del,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    /// The line's number in the old text (context and removed lines).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old: Option<usize>,
    /// The line's number in the new text (context and added lines).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new: Option<usize>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffHunk {
    /// `@@ -a,b +c,d @@`
    pub header: String,
    pub lines: Vec<DiffLine>,
}

/// A change to one file. The agent gives the text `before` and `after`; the hunks are worked out
/// here (`trimmed`) and only they are stored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffView {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    #[default]
    Web,
    File,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceItem {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Where it came from: a site's domain, or "attachment · read by Maya".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default)]
    pub kind: SourceKind,
}

/// What an answer leaned on, numbered like the `[1]` citations in the reply's text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourcesView {
    pub items: Vec<SourceItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionItem {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// What it costs, counted into the running total of what's picked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub selected: bool,
}

/// Several of many: the person ticks the ones they want.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionView {
    pub question: String,
    pub items: Vec<SelectionItem>,
    /// What confirming does, e.g. "Track these".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirm_label: Option<String>,
    /// What the total is per, e.g. "month".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CartItem {
    pub name: String,
    /// Size or pack, e.g. "1 tray · 10 eggs".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default = "one")]
    pub quantity: u32,
    /// The unit price in whole units, as read from the store. Left out when it couldn't be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    /// False when the store is out of it.
    #[serde(default = "yes", skip_serializing_if = "Clone::clone")]
    pub available: bool,
    /// What to get instead, when it's out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swap: Option<String>,
}

fn one() -> u32 {
    1
}
fn yes() -> bool {
    true
}

/// A basket at a store: the person checks out there; Nomi doesn't buy anything.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CartView {
    pub title: String,
    pub store: String,
    pub items: Vec<CartItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery: Option<f64>,
    pub checkout_url: String,
    /// Where and when the prices were read, e.g. "Prices from Sayurbox at 14:05".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Every view, tagged by `view` (`{"view": "timeline", "events": [...]}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "view", rename_all = "snake_case")]
pub enum View {
    List(ListView),
    Table(TableView),
    Comparison(ComparisonView),
    Stepper(StepperView),
    Timeline(TimelineView),
    Card(CardView),
    Chart(ChartView),
    Answer(AnswerView),
    Suggestions(SuggestionsView),
    Decision(DecisionView),
    Form(FormView),
    Link(LinkView),
    Task(TaskView),
    Shell(ShellView),
    Code(CodeView),
    Diff(DiffView),
    Sources(SourcesView),
    Selection(SelectionView),
    Cart(CartView),
    Boundary(BoundaryView),
    QuickReplies(QuickRepliesView),
}

/// One line of the `show` tool's description per view: when to use it, and its fields.
pub struct ViewSpec {
    pub name: &'static str,
    pub use_for: &'static str,
    pub fields: &'static str,
}

pub const CATALOG: &[ViewSpec] = &[
    ViewSpec { name: "list", use_for: "several items of the same kind", fields: "title?, style?: bullet|numbered|check, items: [{title, detail?, meta?, format?, done?, url?}]" },
    ViewSpec { name: "table", use_for: "records with several fields each", fields: "title?, columns: [{key, label, format?}], rows: [{<key>: value}]" },
    ViewSpec { name: "comparison", use_for: "a few options side by side across criteria", fields: "title?, options: [name], rows: [{label, values: [one per option]}], recommended?: option index, verdict?" },
    ViewSpec { name: "stepper", use_for: "steps of a process and where it stands", fields: "title?, steps: [{title, detail?, status?: done|current|upcoming|failed|skipped}]" },
    ViewSpec { name: "timeline", use_for: "things in time order (a schedule, a history, an itinerary)", fields: "title?, events: [{at: ISO date/time or label, title, detail?}]" },
    ViewSpec { name: "card", use_for: "one thing with its key facts", fields: "eyebrow?, title, subtitle?, highlight?: {label, value, format?}, fields?: [{label, value, format?}], body?, link?: {label, url (or an app page like /money)}, actions?: [{label, message}], tone?" },
    ViewSpec { name: "chart", use_for: "numbers to see at a glance: trends, shares, sizes", fields: "title?, chart?: bar|line|donut, labels: [..], series: [{name, values: [one number per label]}], format?" },
    ViewSpec { name: "answer", use_for: "one result to stand out (a total, a yes/no, a date)", fields: "label?, value, format?, detail?, tone?" },
    ViewSpec { name: "suggestions", use_for: "next things the person might ask, to tap", fields: "title?, items: [message in their voice] (1-6)" },
    ViewSpec { name: "decision", use_for: "a choice for the person between 2-5 options", fields: "question, options: [{label, detail?, pros?, cons?, recommended?}]" },
    ViewSpec { name: "form", use_for: "several details to collect at once", fields: "title?, intro?, fields: [{key, label, type?: text|textarea|number|date|time|select|multiselect|checkbox, required?, options?, placeholder?, default?}], submit_label?" },
    ViewSpec { name: "link", use_for: "a page worth opening, as a preview", fields: "url, title?, description?, image?, site? (left-out details are read from the page)" },
    ViewSpec { name: "task", use_for: "work going on in the background", fields: "title, status?: queued|running|done|failed, detail?, progress?: 0-1, agent?, eta? (your rough estimate, e.g. \"10 min\")" },
    ViewSpec { name: "shell", use_for: "a command and what it printed", fields: "command, output?, exit_code?, cwd?" },
    ViewSpec { name: "code", use_for: "code to read or copy (a snippet, a query, a config)", fields: "code, language?, file?" },
    ViewSpec { name: "diff", use_for: "a change to one file, line by line", fields: "path, before, after (the whole text before and after the change)" },
    ViewSpec { name: "sources", use_for: "what an answer leaned on, numbered like the [1] [2] citations in your text", fields: "items: [{title, url?, detail?, kind?: web|file}]" },
    ViewSpec { name: "selection", use_for: "several of many for the person to tick (subscriptions to track, items to keep)", fields: "question, items: [{name, detail?, amount?, selected?}], confirm_label?, per? (what the total is per, e.g. month)" },
    ViewSpec { name: "boundary", use_for: "something you won't or can't do: say why plainly and offer what you can do instead", fields: "title, reason, alternatives?: [{title, detail?, agent?, message? (sent when picked), url?}] (0-4)" },
    ViewSpec { name: "quick_replies", use_for: "short answers to the question you just asked, one tap each", fields: "replies: [answer in their voice] (2-6, a few words each)" },
    ViewSpec { name: "cart", use_for: "a basket at a store, only with prices you actually read from the store; the person checks out there", fields: "title, store, items: [{name, detail?, quantity?, price?, available?, swap?}], delivery?, checkout_url, note? (where and when the prices were read)" },
];

pub fn show_tool_definition() -> nomi_llm::ToolDefinition {
    let mut description = String::from(
        "Show something in the chat as a view instead of plain text, when its shape helps: items, records, \
         steps, times, numbers, choices. Plain text stays best for explanations and short answers; don't use a \
         view for a sentence or two. Don't repeat what the view shows in your reply; add only what it doesn't \
         say: a view can be the whole reply, or come with a line of text before or after it. Views \
         that ask (suggestions, decision, form, card actions) send the person's answer as their next \
         message. `format` is one of text|number|money|percent|date|date_time (money in whole units, dates in \
         ISO 8601); `tone` is neutral|positive|negative|warning. Views and their fields (? = optional):",
    );
    for spec in CATALOG {
        description.push_str(&format!("\n- {}: {}. {}", spec.name, spec.use_for, spec.fields));
    }
    nomi_llm::ToolDefinition {
        name: SHOW_TOOL_NAME.to_string(),
        description,
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "view": {"type": "string", "enum": CATALOG.iter().map(|s| s.name).collect::<Vec<_>>()},
                "spec": {"type": "object", "description": "The view's fields, as listed for that view"}
            },
            "required": ["view", "spec"]
        }),
    }
}

/// Reads a `show` call into a view and checks it. The fields may sit in `spec` or, if a model
/// flattens them, next to `view`.
pub fn parse_show_input(input: &Value) -> Result<View, String> {
    let name = input.get("view").and_then(|v| v.as_str()).ok_or("view is required")?;
    if !CATALOG.iter().any(|s| s.name == name) {
        let names: Vec<_> = CATALOG.iter().map(|s| s.name).collect();
        return Err(format!("there is no {name} view; use one of {}", names.join(", ")));
    }
    let mut fields = Map::new();
    if let Some(obj) = input.as_object() {
        for (k, v) in obj {
            if k != "spec" && k != "view" {
                fields.insert(k.clone(), v.clone());
            }
        }
    }
    match input.get("spec") {
        Some(Value::Object(spec)) => fields.extend(spec.clone()),
        // Some models send nested objects as a JSON string.
        Some(Value::String(text)) => match serde_json::from_str::<Value>(text) {
            Ok(Value::Object(spec)) => fields.extend(spec),
            _ => return Err("spec must be an object with the view's fields".to_string()),
        },
        _ => {}
    }
    fields.insert("view".to_string(), Value::String(name.to_string()));
    let view: View = serde_json::from_value(Value::Object(fields)).map_err(|e| format!("the {name} view's fields don't fit: {e}"))?;
    view.check()?;
    Ok(view)
}

fn count(what: &str, n: usize, min: usize, max: usize) -> Result<(), String> {
    if n < min || n > max {
        return Err(format!("{what}: give {min} to {max}, not {n}"));
    }
    Ok(())
}

fn not_blank(what: &str, s: &str) -> Result<(), String> {
    if s.trim().is_empty() {
        return Err(format!("{what} can't be empty"));
    }
    Ok(())
}

fn web_url(what: &str, url: &str) -> Result<(), String> {
    let lower = url.trim().to_ascii_lowercase();
    if !(lower.starts_with("https://") || lower.starts_with("http://")) {
        return Err(format!("{what} must be an http(s) address"));
    }
    Ok(())
}

/// A link a card opens: a web address, or one of the app's own pages ("/money").
fn card_url(what: &str, url: &str) -> Result<(), String> {
    let url = url.trim();
    if url.starts_with('/') && !url.starts_with("//") {
        return Ok(());
    }
    web_url(what, url)
}

fn keep_end(text: &str, max: usize) -> String {
    let total = text.chars().count();
    if total <= max {
        return text.to_string();
    }
    let tail: String = text.chars().skip(total - max).collect();
    format!("…{tail}")
}

impl View {
    /// The view's name, as in `CATALOG`.
    pub fn name(&self) -> &'static str {
        match self {
            View::List(_) => "list",
            View::Table(_) => "table",
            View::Comparison(_) => "comparison",
            View::Stepper(_) => "stepper",
            View::Timeline(_) => "timeline",
            View::Card(_) => "card",
            View::Chart(_) => "chart",
            View::Answer(_) => "answer",
            View::Suggestions(_) => "suggestions",
            View::Decision(_) => "decision",
            View::Form(_) => "form",
            View::Link(_) => "link",
            View::Task(_) => "task",
            View::Shell(_) => "shell",
            View::Code(_) => "code",
            View::Diff(_) => "diff",
            View::Sources(_) => "sources",
            View::Selection(_) => "selection",
            View::Cart(_) => "cart",
            View::Boundary(_) => "boundary",
            View::QuickReplies(_) => "quick_replies",
        }
    }

    /// Whether the data makes a view worth showing. Errors are written for the agent to fix.
    pub fn check(&self) -> Result<(), String> {
        match self {
            View::List(v) => {
                count("list items", v.items.len(), 1, MAX_ITEMS)?;
                for item in &v.items {
                    not_blank("a list item's title", &item.title)?;
                    if let Some(url) = &item.url {
                        web_url("a list item's url", url)?;
                    }
                }
            }
            View::Table(v) => {
                count("table columns", v.columns.len(), 1, 12)?;
                count("table rows", v.rows.len(), 1, MAX_ITEMS * 2)?;
                for column in &v.columns {
                    not_blank("a column's key", &column.key)?;
                }
            }
            View::Comparison(v) => {
                count("comparison options", v.options.len(), 2, 5)?;
                count("comparison rows", v.rows.len(), 1, 20)?;
                for row in &v.rows {
                    if row.values.len() != v.options.len() {
                        return Err(format!("comparison row {:?} has {} values for {} options", row.label, row.values.len(), v.options.len()));
                    }
                }
                if v.recommended.is_some_and(|i| i >= v.options.len()) {
                    return Err("recommended must be the index of one of the options".to_string());
                }
            }
            View::Stepper(v) => {
                count("steps", v.steps.len(), 1, MAX_ITEMS)?;
                for step in &v.steps {
                    not_blank("a step's title", &step.title)?;
                }
            }
            View::Timeline(v) => {
                count("timeline events", v.events.len(), 1, MAX_ITEMS)?;
                for event in &v.events {
                    not_blank("an event's title", &event.title)?;
                    not_blank("an event's at", &event.at)?;
                }
            }
            View::Card(v) => {
                not_blank("a card's title", &v.title)?;
                count("card fields", v.fields.len(), 0, 16)?;
                count("card actions", v.actions.len(), 0, 4)?;
                if let Some(link) = &v.link {
                    card_url("a card's link", &link.url)?;
                }
                for action in &v.actions {
                    not_blank("an action's message", &action.message)?;
                }
            }
            View::Chart(v) => {
                count("chart labels", v.labels.len(), 1, MAX_ITEMS)?;
                count("chart series", v.series.len(), 1, if v.chart == ChartKind::Donut { 1 } else { 6 })?;
                for series in &v.series {
                    if series.values.len() != v.labels.len() {
                        return Err(format!("series {:?} has {} values for {} labels", series.name, series.values.len(), v.labels.len()));
                    }
                    if series.values.iter().any(|n| !n.is_finite()) {
                        return Err("chart values must be numbers".to_string());
                    }
                    if v.chart == ChartKind::Donut && series.values.iter().any(|n| *n < 0.0) {
                        return Err("a donut's values can't be negative".to_string());
                    }
                }
            }
            View::Answer(v) => {
                if let Scalar::Text(text) = &v.value {
                    not_blank("an answer's value", text)?;
                }
            }
            View::Suggestions(v) => {
                count("suggestions", v.items.len(), 1, 6)?;
                for item in &v.items {
                    not_blank("a suggestion", item)?;
                }
            }
            View::Decision(v) => {
                not_blank("a decision's question", &v.question)?;
                count("decision options", v.options.len(), 2, 5)?;
            }
            View::Form(v) => {
                count("form fields", v.fields.len(), 1, 12)?;
                let mut keys = std::collections::HashSet::new();
                for field in &v.fields {
                    not_blank("a form field's key", &field.key)?;
                    if !keys.insert(field.key.as_str()) {
                        return Err(format!("two form fields share the key {:?}", field.key));
                    }
                    if matches!(field.kind, FieldKind::Select | FieldKind::Multiselect) {
                        count(&format!("{:?}'s options", field.key), field.options.len(), 2, 20)?;
                    }
                }
            }
            View::Link(v) => web_url("a link's url", &v.url)?,
            View::Task(v) => {
                not_blank("a task's title", &v.title)?;
                if v.progress.is_some_and(|p| !(0.0..=1.0).contains(&p)) {
                    return Err("progress is a fraction from 0 to 1".to_string());
                }
            }
            View::Shell(v) => not_blank("a shell view's command", &v.command)?,
            View::Code(v) => {
                not_blank("a code view's code", &v.code)?;
                if v.code.chars().count() > MAX_CODE {
                    return Err(format!("code: at most {MAX_CODE} characters; show the part that matters"));
                }
            }
            View::Diff(v) => {
                not_blank("a diff's path", &v.path)?;
                if v.hunks.is_empty() {
                    let (Some(before), Some(after)) = (&v.before, &v.after) else {
                        return Err("a diff needs the file's text before and after".to_string());
                    };
                    if before == after {
                        return Err("before and after are the same: nothing changed".to_string());
                    }
                    if before.lines().count() > MAX_DIFF_LINES || after.lines().count() > MAX_DIFF_LINES {
                        return Err(format!("a diff: at most {MAX_DIFF_LINES} lines on each side; show the part that changed"));
                    }
                }
            }
            View::Sources(v) => {
                count("sources", v.items.len(), 1, 12)?;
                for item in &v.items {
                    not_blank("a source's title", &item.title)?;
                    if let Some(url) = &item.url {
                        web_url("a source's url", url)?;
                    }
                }
            }
            View::Selection(v) => {
                not_blank("a selection's question", &v.question)?;
                count("selection items", v.items.len(), 2, 20)?;
                for item in &v.items {
                    not_blank("a selection item's name", &item.name)?;
                    if item.amount.is_some_and(|a| !a.is_finite()) {
                        return Err("an amount must be a number".to_string());
                    }
                }
            }
            View::Boundary(v) => {
                not_blank("a boundary's title", &v.title)?;
                not_blank("a boundary's reason", &v.reason)?;
                count("alternatives", v.alternatives.len(), 0, 4)?;
                for alternative in &v.alternatives {
                    not_blank("an alternative's title", &alternative.title)?;
                    if let Some(url) = &alternative.url {
                        card_url("an alternative's url", url)?;
                    }
                }
            }
            View::QuickReplies(v) => {
                count("quick replies", v.replies.len(), 2, 6)?;
                for reply in &v.replies {
                    not_blank("a quick reply", reply)?;
                    if reply.chars().count() > 60 {
                        return Err("quick replies are a few words each; use suggestions for longer ones".to_string());
                    }
                }
            }
            View::Cart(v) => {
                not_blank("a cart's title", &v.title)?;
                not_blank("a cart's store", &v.store)?;
                count("cart items", v.items.len(), 1, 30)?;
                web_url("a cart's checkout_url", &v.checkout_url)?;
                for item in &v.items {
                    not_blank("a cart item's name", &item.name)?;
                    if item.price.is_some_and(|p| !p.is_finite() || p < 0.0) || v.delivery.is_some_and(|p| !p.is_finite() || p < 0.0) {
                        return Err("prices are amounts of 0 or more".to_string());
                    }
                }
            }
        }
        Ok(())
    }

    /// Trims what a view may hold too much of (a command's whole log), before it's stored.
    pub fn trimmed(mut self) -> Self {
        match &mut self {
            View::Shell(shell) => {
                if let Some(output) = &shell.output {
                    shell.output = Some(keep_end(output, MAX_SHELL_OUTPUT));
                }
            }
            // The hunks are what's drawn; the whole texts would only weigh down the message.
            View::Diff(diff) => {
                if diff.hunks.is_empty() {
                    if let (Some(before), Some(after)) = (&diff.before, &diff.after) {
                        diff.hunks = diff_hunks(before, after, DIFF_CONTEXT);
                    }
                }
                diff.before = None;
                diff.after = None;
            }
            _ => {}
        }
        self
    }

    /// The view as plain text: the message's text, which history, notifications and the model's
    /// own context read where the view itself isn't drawn.
    pub fn plain_text(&self) -> String {
        let mut out = Vec::<String>::new();
        let heading = |out: &mut Vec<String>, title: &Option<String>| {
            if let Some(title) = title.as_deref().filter(|t| !t.trim().is_empty()) {
                out.push(title.to_string());
            }
        };
        match self {
            View::List(v) => {
                heading(&mut out, &v.title);
                for (i, item) in v.items.iter().enumerate() {
                    let mark = match v.style {
                        ListStyle::Bullet => "-".to_string(),
                        ListStyle::Numbered => format!("{}.", i + 1),
                        ListStyle::Check => if item.done == Some(true) { "[x]" } else { "[ ]" }.to_string(),
                    };
                    let mut line = format!("{mark} {}", item.title);
                    if let Some(detail) = &item.detail {
                        line.push_str(&format!(" — {detail}"));
                    }
                    if let Some(meta) = &item.meta {
                        line.push_str(&format!(" ({meta})"));
                    }
                    out.push(line);
                }
            }
            View::Table(v) => {
                heading(&mut out, &v.title);
                out.push(format!("| {} |", v.columns.iter().map(|c| c.label.as_str()).collect::<Vec<_>>().join(" | ")));
                out.push(format!("|{}", " --- |".repeat(v.columns.len())));
                for row in &v.rows {
                    let cells: Vec<String> = v
                        .columns
                        .iter()
                        .map(|c| match row.get(&c.key) {
                            Some(Value::String(s)) => s.clone(),
                            Some(Value::Null) | None => String::new(),
                            Some(other) => other.to_string(),
                        })
                        .collect();
                    out.push(format!("| {} |", cells.join(" | ")));
                }
            }
            View::Comparison(v) => {
                heading(&mut out, &v.title);
                out.push(v.options.join(" vs "));
                for row in &v.rows {
                    out.push(format!("- {}: {}", row.label, row.values.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" | ")));
                }
                if let Some(option) = v.recommended.and_then(|i| v.options.get(i)) {
                    out.push(format!("Recommended: {option}"));
                }
                if let Some(verdict) = &v.verdict {
                    out.push(verdict.clone());
                }
            }
            View::Stepper(v) => {
                heading(&mut out, &v.title);
                for (i, step) in v.steps.iter().enumerate() {
                    let mark = match step.status {
                        StepStatus::Done => "✓",
                        StepStatus::Current => "→",
                        StepStatus::Upcoming => "○",
                        StepStatus::Failed => "✗",
                        StepStatus::Skipped => "–",
                    };
                    let detail = step.detail.as_deref().map(|d| format!(" — {d}")).unwrap_or_default();
                    out.push(format!("{}. {mark} {}{detail}", i + 1, step.title));
                }
            }
            View::Timeline(v) => {
                heading(&mut out, &v.title);
                for event in &v.events {
                    let detail = event.detail.as_deref().map(|d| format!(" — {d}")).unwrap_or_default();
                    out.push(format!("- {}: {}{detail}", event.at, event.title));
                }
            }
            View::Card(v) => {
                if let Some(eyebrow) = &v.eyebrow {
                    out.push(eyebrow.clone());
                }
                out.push(v.title.clone());
                if let Some(subtitle) = &v.subtitle {
                    out.push(subtitle.clone());
                }
                for field in v.highlight.iter().chain(&v.fields) {
                    out.push(format!("{}: {}", field.label, field.value));
                }
                if let Some(body) = &v.body {
                    out.push(body.clone());
                }
                if let Some(link) = &v.link {
                    out.push(format!("{}: {}", link.label, link.url));
                }
            }
            View::Chart(v) => {
                heading(&mut out, &v.title);
                for series in &v.series {
                    let points: Vec<String> = v.labels.iter().zip(&series.values).map(|(l, n)| format!("{l} {}", Scalar::Number(*n))).collect();
                    out.push(format!("{}: {}", series.name, points.join(", ")));
                }
            }
            View::Answer(v) => {
                out.push(match &v.label {
                    Some(label) => format!("{label}: {}", v.value),
                    None => v.value.to_string(),
                });
                if let Some(detail) = &v.detail {
                    out.push(detail.clone());
                }
            }
            View::Suggestions(v) => {
                heading(&mut out, &v.title);
                out.extend(v.items.iter().map(|s| format!("- {s}")));
            }
            View::Decision(v) => {
                out.push(v.question.clone());
                for option in &v.options {
                    let star = if option.recommended { " (recommended)" } else { "" };
                    let detail = option.detail.as_deref().map(|d| format!(" — {d}")).unwrap_or_default();
                    out.push(format!("- {}{star}{detail}", option.label));
                }
            }
            View::Form(v) => {
                heading(&mut out, &v.title);
                if let Some(intro) = &v.intro {
                    out.push(intro.clone());
                }
                out.extend(v.fields.iter().map(|f| format!("- {}{}", f.label, if f.required { " *" } else { "" })));
            }
            View::Link(v) => {
                out.push(match &v.title {
                    Some(title) => format!("{title} — {}", v.url),
                    None => v.url.clone(),
                });
                if let Some(description) = &v.description {
                    out.push(description.clone());
                }
            }
            View::Task(v) => {
                let status = match v.status {
                    TaskStatus::Queued => "queued",
                    TaskStatus::Running => "running",
                    TaskStatus::Done => "done",
                    TaskStatus::Failed => "failed",
                    TaskStatus::Cancelled => "cancelled",
                };
                let detail = v.detail.as_deref().map(|d| format!(" — {d}")).unwrap_or_default();
                out.push(format!("{} ({status}){detail}", v.title));
            }
            View::Shell(v) => {
                out.push(format!("$ {}", v.command));
                if let Some(output) = &v.output {
                    out.push(output.clone());
                }
                if let Some(code) = v.exit_code.filter(|c| *c != 0) {
                    out.push(format!("(exit {code})"));
                }
            }
            View::Code(v) => {
                if let Some(file) = &v.file {
                    out.push(file.clone());
                }
                out.push(format!("```{}\n{}\n```", v.language.as_deref().unwrap_or(""), v.code.trim_end()));
            }
            View::Diff(v) => {
                out.push(format!("Changes to {}", v.path));
                let hunks = if v.hunks.is_empty() {
                    match (&v.before, &v.after) {
                        (Some(before), Some(after)) => diff_hunks(before, after, DIFF_CONTEXT),
                        _ => Vec::new(),
                    }
                } else {
                    v.hunks.clone()
                };
                for hunk in &hunks {
                    out.push(hunk.header.clone());
                    for line in &hunk.lines {
                        let sign = match line.kind {
                            DiffLineKind::Context => ' ',
                            DiffLineKind::Add => '+',
                            DiffLineKind::Del => '-',
                        };
                        out.push(format!("{sign}{}", line.text));
                    }
                }
            }
            View::Sources(v) => {
                out.push("Sources".to_string());
                for (i, item) in v.items.iter().enumerate() {
                    let url = item.url.as_deref().map(|u| format!(" — {u}")).unwrap_or_default();
                    out.push(format!("[{}] {}{url}", i + 1, item.title));
                }
            }
            View::Selection(v) => {
                out.push(v.question.clone());
                for item in &v.items {
                    let mark = if item.selected { "[x]" } else { "[ ]" };
                    let amount = item.amount.map(|a| format!(" ({})", Scalar::Number(a))).unwrap_or_default();
                    out.push(format!("{mark} {}{amount}", item.name));
                }
            }
            View::Boundary(v) => {
                out.push(v.title.clone());
                out.push(v.reason.clone());
                for alternative in &v.alternatives {
                    let detail = alternative.detail.as_deref().map(|d| format!(" — {d}")).unwrap_or_default();
                    out.push(format!("- {}{detail}", alternative.title));
                }
            }
            View::QuickReplies(v) => out.extend(v.replies.iter().map(|r| format!("- {r}"))),
            View::Cart(v) => {
                out.push(format!("{} — {}", v.title, v.store));
                for item in &v.items {
                    let price = match (item.available, item.price) {
                        (false, _) => " (out of stock)".to_string(),
                        (true, Some(p)) => format!(" @ {}", Scalar::Number(p)),
                        (true, None) => String::new(),
                    };
                    out.push(format!("- {} × {}{price}", item.quantity, item.name));
                }
                if let Some(delivery) = v.delivery {
                    out.push(format!("Delivery: {}", Scalar::Number(delivery)));
                }
                out.push(format!("Check out: {}", v.checkout_url));
            }
        }
        out.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One example per view, shared with the frontend's tests (frontend/src/lib/views).
    const EXAMPLES: &str = include_str!("../../../../frontend/src/lib/views/examples.json");

    fn examples() -> Vec<Value> {
        serde_json::from_str::<Vec<Value>>(EXAMPLES).expect("examples.json is a list of views")
    }

    #[test]
    fn every_view_has_a_catalog_line_and_an_example_that_parses() {
        let examples = examples();
        for spec in CATALOG {
            let example = examples.iter().find(|e| e["view"] == spec.name).unwrap_or_else(|| panic!("no example for the {} view", spec.name));
            let view: View = serde_json::from_value(example.clone()).unwrap_or_else(|e| panic!("{}: {e}", spec.name));
            assert_eq!(view.name(), spec.name);
            view.check().unwrap_or_else(|e| panic!("{}: {e}", spec.name));
            assert!(!view.plain_text().is_empty(), "{} has no plain text", spec.name);
            // What is stored reads back the same.
            let again: View = serde_json::from_value(serde_json::to_value(&view).unwrap()).unwrap();
            assert_eq!(again, view, "{} doesn't round-trip", spec.name);
        }
        assert_eq!(examples.len(), CATALOG.len(), "an example without a catalog line");
    }

    #[test]
    fn a_card_may_link_to_an_app_page_but_not_elsewhere_off_the_web() {
        let card = |url: &str| parse_show_input(&serde_json::json!({"view": "card", "spec": {"title": "t", "link": {"label": "Open", "url": url}}}));
        assert!(card("/money").is_ok());
        assert!(card("https://example.com").is_ok());
        assert!(card("//evil.example").is_err());
        assert!(card("javascript:alert(1)").is_err());
    }

    #[test]
    fn show_reads_fields_from_spec_from_a_json_string_or_flattened() {
        let nested = parse_show_input(&serde_json::json!({"view": "answer", "spec": {"value": 42, "label": "Total"}})).unwrap();
        let text = parse_show_input(&serde_json::json!({"view": "answer", "spec": "{\"value\": 42, \"label\": \"Total\"}"})).unwrap();
        let flat = parse_show_input(&serde_json::json!({"view": "answer", "value": 42, "label": "Total"})).unwrap();
        assert_eq!(nested, text);
        assert_eq!(nested, flat);
        assert_eq!(nested.plain_text(), "Total: 42");
    }

    #[test]
    fn show_refuses_unknown_views_and_data_that_does_not_fit() {
        assert!(parse_show_input(&serde_json::json!({"view": "carousel", "spec": {}})).unwrap_err().contains("no carousel view"));
        assert!(parse_show_input(&serde_json::json!({"view": "list", "spec": {"items": []}})).unwrap_err().contains("list items"));
        let mismatched = serde_json::json!({"view": "chart", "spec": {"labels": ["a", "b"], "series": [{"name": "x", "values": [1]}]}});
        assert!(parse_show_input(&mismatched).unwrap_err().contains("1 values for 2 labels"));
        let bad_link = serde_json::json!({"view": "link", "spec": {"url": "javascript:alert(1)"}});
        assert!(parse_show_input(&bad_link).unwrap_err().contains("http(s)"));
        let select = serde_json::json!({"view": "form", "spec": {"fields": [{"key": "size", "label": "Size", "type": "select", "options": ["S"]}]}});
        assert!(parse_show_input(&select).unwrap_err().contains("options"));
    }

    #[test]
    fn extra_fields_are_ignored_not_refused() {
        let view = parse_show_input(&serde_json::json!({"view": "task", "spec": {"title": "Deploy", "status": "running", "emoji": "🚀"}})).unwrap();
        assert_eq!(view.plain_text(), "Deploy (running)");
    }

    #[test]
    fn a_shell_view_keeps_the_end_of_a_long_output() {
        let long = format!("{}DONE", "x".repeat(MAX_SHELL_OUTPUT * 2));
        let view = View::Shell(ShellView { command: "npm test".into(), output: Some(long), exit_code: Some(0), cwd: None }).trimmed();
        let View::Shell(shell) = view else { unreachable!() };
        let output = shell.output.unwrap();
        assert!(output.ends_with("DONE") && output.chars().count() == MAX_SHELL_OUTPUT + 1);
    }

    #[test]
    fn the_tool_lists_every_view() {
        let tool = show_tool_definition();
        for spec in CATALOG {
            assert!(tool.description.contains(&format!("- {}:", spec.name)), "{} missing from the description", spec.name);
        }
    }

    #[test]
    fn a_diff_is_worked_out_from_before_and_after_and_only_its_hunks_are_kept() {
        let before = "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n";
        let after = "a\nb\nC\nd\ne\nf\ng\nh\ni\nj\nk\n";
        let view = parse_show_input(&serde_json::json!({"view": "diff", "spec": {"path": "x.txt", "before": before, "after": after}})).unwrap().trimmed();
        let View::Diff(diff) = &view else { unreachable!() };
        assert!(diff.before.is_none() && diff.after.is_none());
        assert_eq!(diff.hunks.len(), 2, "{:#?}", diff.hunks);
        assert_eq!(diff.hunks[0].header, "@@ -1,6 +1,6 @@");
        let changed: Vec<_> = diff.hunks[0].lines.iter().filter(|l| l.kind != DiffLineKind::Context).map(|l| (l.kind, l.old, l.new, l.text.as_str())).collect();
        assert_eq!(changed, vec![(DiffLineKind::Del, Some(3), None, "c"), (DiffLineKind::Add, None, Some(3), "C")]);
        assert_eq!(diff.hunks[1].lines.last().unwrap().text, "k");
        assert!(view.plain_text().contains("-c\n+C"));
        // Stored, it reads back the same.
        let again: View = serde_json::from_value(serde_json::to_value(&view).unwrap()).unwrap();
        assert_eq!(again, view);
    }

    #[test]
    fn a_diff_without_a_change_or_its_texts_is_refused() {
        assert!(parse_show_input(&serde_json::json!({"view": "diff", "spec": {"path": "x", "before": "a", "after": "a"}})).unwrap_err().contains("nothing changed"));
        assert!(parse_show_input(&serde_json::json!({"view": "diff", "spec": {"path": "x"}})).unwrap_err().contains("before and after"));
    }

    #[test]
    fn a_cart_checks_out_on_the_web_and_items_default_to_one_in_stock() {
        let cart = parse_show_input(&serde_json::json!({"view": "cart", "spec": {"title": "Groceries", "store": "Sayurbox", "checkout_url": "https://www.sayurbox.com", "items": [{"name": "Eggs", "price": 29000}]}})).unwrap();
        let View::Cart(cart) = cart else { unreachable!() };
        assert_eq!((cart.items[0].quantity, cart.items[0].available), (1, true));
        let bad = serde_json::json!({"view": "cart", "spec": {"title": "G", "store": "S", "checkout_url": "javascript:x", "items": [{"name": "Eggs"}]}});
        assert!(parse_show_input(&bad).unwrap_err().contains("http(s)"));
    }
}
