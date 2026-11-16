//! Where an app or site stands on its way from the first request to a running build, and who may
//! pass it on. The order is fixed: Rena finds out what is wanted and writes the plan and the
//! brief, the person agrees, Sora proposes directions, the person picks one, Sora designs it, the
//! person approves a version, Sora writes the handoff, Koda builds.
//!
//! The stage is read from what the chat has saved (plans, briefs, directions, previews,
//! approvals, handoffs, Koda's work), never from what an agent says, so no agent can skip ahead:
//! `handover` refuses a `delegate_to_agent` that comes too early and tells the agent what is
//! still missing, and `preview_allowed` does the same for Sora's first preview.

use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::crew::{CODING_AGENT_TYPE, DESIGN_AGENT_TYPE, PLANNING_AGENT_TYPE};
use crate::prompts;

/// What the chat has saved so far.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Facts {
    /// The chat has a project: it is about building something.
    pub has_project: bool,
    /// When the newest version of the plan was written.
    pub plan_at: Option<DateTime<Utc>>,
    /// When the newest version of the design brief was saved.
    pub brief_at: Option<DateTime<Utc>>,
    /// When the person last wrote in the chat.
    pub person_spoke_at: Option<DateTime<Utc>>,
    /// When Sora last proposed directions.
    pub directions_at: Option<DateTime<Utc>>,
    /// The newest preview version.
    pub preview_version: Option<i32>,
    /// The preview version the person approved.
    pub approved_version: Option<i32>,
    /// The preview version the newest handoff describes.
    pub handoff_version: Option<i32>,
    /// Koda has work queued or in hand in this chat.
    pub building: bool,
    /// Koda has finished work in this chat before.
    pub built: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Rena is finding out what is wanted: no plan yet.
    Discovery,
    /// The plan is written and the person hasn't answered it.
    PlanReview,
    /// The person answered the plan: the design (or, without Sora, the build) is next.
    PlanAgreed,
    /// Sora proposed directions and the person hasn't picked.
    DirectionReview,
    /// The person answered the directions: Sora builds the design.
    Designing,
    /// A preview is saved and no version is approved.
    DesignReview,
    /// The person approved a version; its handoff isn't written.
    Approved,
    /// The handoff for the approved version is written; Koda hasn't started.
    ReadyToBuild,
    /// Koda is building.
    Building,
    /// Koda has built it; changes go straight to whoever makes them.
    Built,
}

impl Facts {
    fn answered(&self, since: Option<DateTime<Utc>>) -> bool {
        matches!((self.person_spoke_at, since), (Some(spoke), Some(since)) if spoke > since)
    }

    /// The stage these facts add up to. Without Sora (`design_on` false) the design stages don't
    /// exist: an agreed plan is ready to build.
    pub fn stage(&self, design_on: bool) -> Stage {
        if self.building {
            return Stage::Building;
        }
        if self.built {
            return Stage::Built;
        }
        if design_on {
            match (self.approved_version, self.handoff_version) {
                (Some(approved), Some(handed)) if approved == handed => return Stage::ReadyToBuild,
                (Some(_), _) => return Stage::Approved,
                _ => {}
            }
            if self.preview_version.is_some() {
                return Stage::DesignReview;
            }
            if self.directions_at.is_some() {
                return if self.answered(self.directions_at) { Stage::Designing } else { Stage::DirectionReview };
            }
        }
        match self.plan_at {
            None => Stage::Discovery,
            Some(_) if self.answered(self.plan_at) => Stage::PlanAgreed,
            Some(_) => Stage::PlanReview,
        }
    }

    /// Whether `from` may hand the chat's work to `to` now. Only the three who carry a build
    /// (Rena, Sora, Koda) are held to the order, only in a chat with a project, and only until
    /// the first build is done: later changes go straight through. Without Sora nothing is
    /// refused.
    pub fn handover(&self, design_on: bool, from: &str, to: &str) -> Result<(), String> {
        if !design_on || !self.has_project || self.built || self.building {
            return Ok(());
        }
        let stage = self.stage(true);
        let refuse = |why: &str| Err(why.to_string());
        match (from, to) {
            (PLANNING_AGENT_TYPE, DESIGN_AGENT_TYPE | CODING_AGENT_TYPE) if stage == Stage::Discovery => refuse(prompts::FLOW_PLAN_FIRST),
            (PLANNING_AGENT_TYPE, DESIGN_AGENT_TYPE | CODING_AGENT_TYPE) if stage == Stage::PlanReview => refuse(prompts::FLOW_PLAN_NOT_ANSWERED),
            (PLANNING_AGENT_TYPE, DESIGN_AGENT_TYPE) if self.brief_at.is_none() => refuse(prompts::FLOW_BRIEF_FIRST),
            // A brief means there is something to design: it goes through Sora before Koda.
            (PLANNING_AGENT_TYPE, CODING_AGENT_TYPE) if self.brief_at.is_some() && stage != Stage::ReadyToBuild => refuse(prompts::FLOW_DESIGN_FIRST),
            (DESIGN_AGENT_TYPE, CODING_AGENT_TYPE) if stage == Stage::Approved => refuse(prompts::FLOW_HANDOFF_FIRST),
            (DESIGN_AGENT_TYPE, CODING_AGENT_TYPE) if stage != Stage::ReadyToBuild => refuse(prompts::FLOW_NOT_APPROVED),
            _ => Ok(()),
        }
    }

    /// Whether Sora may save a preview now: the first one waits until she proposed directions
    /// and the person answered them. Later versions (acting on feedback) always may.
    pub fn preview_allowed(&self) -> Result<(), String> {
        if self.preview_version.is_some() {
            return Ok(());
        }
        match self.directions_at {
            None => Err(prompts::FLOW_DIRECTIONS_FIRST.to_string()),
            Some(_) if !self.answered(self.directions_at) => Err(prompts::FLOW_DIRECTION_NOT_PICKED.to_string()),
            Some(_) => Ok(()),
        }
    }
}

impl Stage {
    /// Who acts now and who waits, as every agent in the chat is told.
    pub fn line(self) -> &'static str {
        match self {
            Stage::Discovery => prompts::FLOW_STAGE_DISCOVERY,
            Stage::PlanReview => prompts::FLOW_STAGE_PLAN_REVIEW,
            Stage::PlanAgreed => prompts::FLOW_STAGE_PLAN_AGREED,
            Stage::DirectionReview => prompts::FLOW_STAGE_DIRECTION_REVIEW,
            Stage::Designing => prompts::FLOW_STAGE_DESIGNING,
            Stage::DesignReview => prompts::FLOW_STAGE_DESIGN_REVIEW,
            Stage::Approved => prompts::FLOW_STAGE_APPROVED,
            Stage::ReadyToBuild => prompts::FLOW_STAGE_READY_TO_BUILD,
            Stage::Building => prompts::FLOW_STAGE_BUILDING,
            Stage::Built => prompts::FLOW_STAGE_BUILT,
        }
    }
}

/// What Rena, Sora and Koda read about the order of a build and where this chat is in it.
pub fn context(stage: Stage) -> String {
    format!("{}\n\n{} {}", prompts::BUILD_FLOW_GUIDE, prompts::FLOW_NOW, stage.line())
}

/// What the chat has saved so far.
pub async fn facts(conn: &mut PgConnection, session_id: Uuid) -> Result<Facts, sqlx::Error> {
    #[allow(clippy::type_complexity)]
    let row: (
        bool,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        bool,
        bool,
    ) = sqlx::query_as(
        "SELECT \
           EXISTS(SELECT 1 FROM projects WHERE session_id = $1), \
           (SELECT max(created_at) FROM agent_plans WHERE session_id = $1), \
           (SELECT max(created_at) FROM design_briefs WHERE session_id = $1), \
           (SELECT max(created_at) FROM messages WHERE session_id = $1 AND sender_channel_identity_id IS NOT NULL AND NOT is_notice), \
           (SELECT max(created_at) FROM design_direction_sets WHERE session_id = $1), \
           (SELECT max(version) FROM design_artifacts WHERE session_id = $1), \
           (SELECT artifact_version FROM design_approvals WHERE session_id = $1 ORDER BY created_at DESC, id DESC LIMIT 1), \
           (SELECT artifact_version FROM design_handoffs WHERE session_id = $1 ORDER BY version DESC LIMIT 1), \
           EXISTS(SELECT 1 FROM agent_delegations WHERE session_id = $1 AND target_agent_type = $2 AND status IN ('pending', 'claimed', 'processing')), \
           EXISTS(SELECT 1 FROM agent_delegations WHERE session_id = $1 AND target_agent_type = $2 AND status = 'completed')",
    )
    .bind(session_id)
    .bind(CODING_AGENT_TYPE)
    .fetch_one(conn)
    .await?;
    let (has_project, plan_at, brief_at, person_spoke_at, directions_at, preview_version, approved_version, handoff_version, building, built) = row;
    Ok(Facts { has_project, plan_at, brief_at, person_spoke_at, directions_at, preview_version, approved_version, handoff_version, building, built })
}

/// `Facts::handover` for this chat and person as things are saved now: the reason when `from`
/// may not hand to `to` yet. A chat that can't be read refuses nothing.
pub async fn handover_refusal(conn: &mut PgConnection, session_id: Uuid, from: &str, to: &str) -> Option<String> {
    if ![PLANNING_AGENT_TYPE, DESIGN_AGENT_TYPE].contains(&from) || ![DESIGN_AGENT_TYPE, CODING_AGENT_TYPE].contains(&to) {
        return None;
    }
    let design_on = nomi_settings::config::agent_on(DESIGN_AGENT_TYPE).await;
    facts(conn, session_id).await.ok()?.handover(design_on, from, to).err()
}

/// `context` for this chat, for a chat with a project. `None` elsewhere.
pub async fn context_for(conn: &mut PgConnection, session_id: Uuid) -> Option<String> {
    let facts = facts(conn, session_id).await.ok()?;
    if !facts.has_project {
        return None;
    }
    Some(context(facts.stage(nomi_settings::config::agent_on(DESIGN_AGENT_TYPE).await)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn at(minutes: i64) -> Option<DateTime<Utc>> {
        Some(DateTime::<Utc>::from_timestamp(1_800_000_000, 0).unwrap() + Duration::minutes(minutes))
    }

    /// A chat with a project where the person wrote at minute 0.
    fn asked() -> Facts {
        Facts { has_project: true, person_spoke_at: at(0), ..Facts::default() }
    }

    #[test]
    fn a_plan_is_reviewed_until_the_person_answers_it() {
        assert_eq!(asked().stage(true), Stage::Discovery);
        let written = Facts { plan_at: at(1), ..asked() };
        assert_eq!(written.stage(true), Stage::PlanReview, "written in the turn the person asked in");
        let answered = Facts { person_spoke_at: at(2), ..written.clone() };
        assert_eq!(answered.stage(true), Stage::PlanAgreed);
        let rewritten = Facts { plan_at: at(3), ..answered };
        assert_eq!(rewritten.stage(true), Stage::PlanReview, "a new version waits for an answer again");
    }

    #[test]
    fn the_design_moves_from_directions_to_an_approved_handoff() {
        let agreed = Facts { plan_at: at(1), brief_at: at(1), person_spoke_at: at(2), ..asked() };
        let proposed = Facts { directions_at: at(3), ..agreed };
        assert_eq!(proposed.stage(true), Stage::DirectionReview);
        let picked = Facts { person_spoke_at: at(4), ..proposed };
        assert_eq!(picked.stage(true), Stage::Designing);
        let previewed = Facts { preview_version: Some(2), ..picked };
        assert_eq!(previewed.stage(true), Stage::DesignReview);
        let approved = Facts { approved_version: Some(2), ..previewed };
        assert_eq!(approved.stage(true), Stage::Approved);
        let stale = Facts { handoff_version: Some(1), ..approved.clone() };
        assert_eq!(stale.stage(true), Stage::Approved, "a handoff of another version isn't the approved one's");
        let handed = Facts { handoff_version: Some(2), ..approved };
        assert_eq!(handed.stage(true), Stage::ReadyToBuild);
        assert_eq!(Facts { building: true, ..handed.clone() }.stage(true), Stage::Building);
        assert_eq!(Facts { built: true, ..handed }.stage(true), Stage::Built);
    }

    #[test]
    fn without_sora_an_agreed_plan_is_all_there_is() {
        let designed = Facts { plan_at: at(1), person_spoke_at: at(2), directions_at: at(3), preview_version: Some(1), ..asked() };
        assert_eq!(designed.stage(false), Stage::PlanAgreed);
    }

    #[test]
    fn rena_cannot_pass_a_plan_on_before_the_person_answered_it() {
        let none = asked();
        assert_eq!(none.handover(true, PLANNING_AGENT_TYPE, DESIGN_AGENT_TYPE).unwrap_err(), prompts::FLOW_PLAN_FIRST);
        let written = Facts { plan_at: at(1), brief_at: at(1), ..asked() };
        assert_eq!(written.handover(true, PLANNING_AGENT_TYPE, DESIGN_AGENT_TYPE).unwrap_err(), prompts::FLOW_PLAN_NOT_ANSWERED);
        assert_eq!(written.handover(true, PLANNING_AGENT_TYPE, CODING_AGENT_TYPE).unwrap_err(), prompts::FLOW_PLAN_NOT_ANSWERED);
        let answered = Facts { person_spoke_at: at(2), ..written };
        assert_eq!(answered.handover(true, PLANNING_AGENT_TYPE, DESIGN_AGENT_TYPE), Ok(()));
    }

    #[test]
    fn sora_gets_a_brief_and_a_design_goes_through_her_before_koda() {
        let no_brief = Facts { plan_at: at(1), person_spoke_at: at(2), ..asked() };
        assert_eq!(no_brief.handover(true, PLANNING_AGENT_TYPE, DESIGN_AGENT_TYPE).unwrap_err(), prompts::FLOW_BRIEF_FIRST);
        assert_eq!(no_brief.handover(true, PLANNING_AGENT_TYPE, CODING_AGENT_TYPE), Ok(()), "nothing to design: a script goes straight to Koda");
        let with_brief = Facts { brief_at: at(1), ..no_brief };
        assert_eq!(with_brief.handover(true, PLANNING_AGENT_TYPE, CODING_AGENT_TYPE).unwrap_err(), prompts::FLOW_DESIGN_FIRST);
    }

    #[test]
    fn sora_passes_only_an_approved_design_with_its_handoff_to_koda() {
        let previewed = Facts { plan_at: at(1), brief_at: at(1), directions_at: at(3), person_spoke_at: at(4), preview_version: Some(1), ..asked() };
        assert_eq!(previewed.handover(true, DESIGN_AGENT_TYPE, CODING_AGENT_TYPE).unwrap_err(), prompts::FLOW_NOT_APPROVED);
        let approved = Facts { approved_version: Some(1), ..previewed };
        assert_eq!(approved.handover(true, DESIGN_AGENT_TYPE, CODING_AGENT_TYPE).unwrap_err(), prompts::FLOW_HANDOFF_FIRST);
        let handed = Facts { handoff_version: Some(1), ..approved };
        assert_eq!(handed.handover(true, DESIGN_AGENT_TYPE, CODING_AGENT_TYPE), Ok(()));
        assert_eq!(handed.handover(true, PLANNING_AGENT_TYPE, CODING_AGENT_TYPE), Ok(()));
    }

    #[test]
    fn nothing_is_held_back_without_sora_without_a_project_or_after_the_first_build() {
        let early = asked();
        assert_eq!(early.handover(false, PLANNING_AGENT_TYPE, CODING_AGENT_TYPE), Ok(()), "Sora is off: as it always was");
        assert_eq!(Facts { has_project: false, ..early.clone() }.handover(true, PLANNING_AGENT_TYPE, DESIGN_AGENT_TYPE), Ok(()));
        assert_eq!(Facts { built: true, ..early.clone() }.handover(true, PLANNING_AGENT_TYPE, CODING_AGENT_TYPE), Ok(()), "a change to a built app");
        assert_eq!(early.handover(true, "chitchat", CODING_AGENT_TYPE), Ok(()), "only Rena and Sora are held to the order");
        assert_eq!(early.handover(true, DESIGN_AGENT_TYPE, PLANNING_AGENT_TYPE), Ok(()), "Sora may always go back to Rena");
    }

    #[test]
    fn the_first_preview_waits_for_the_person_to_pick_a_direction() {
        assert_eq!(asked().preview_allowed().unwrap_err(), prompts::FLOW_DIRECTIONS_FIRST);
        let proposed = Facts { directions_at: at(1), ..asked() };
        assert_eq!(proposed.preview_allowed().unwrap_err(), prompts::FLOW_DIRECTION_NOT_PICKED);
        let picked = Facts { person_spoke_at: at(2), ..proposed.clone() };
        assert_eq!(picked.preview_allowed(), Ok(()));
        assert_eq!(Facts { preview_version: Some(1), ..proposed }.preview_allowed(), Ok(()), "acting on feedback needs no new pick");
    }

    #[test]
    fn every_agent_reads_the_order_and_where_the_chat_is() {
        let said = context(Stage::DirectionReview);
        assert!(said.starts_with(prompts::BUILD_FLOW_GUIDE) && said.ends_with(prompts::FLOW_STAGE_DIRECTION_REVIEW), "{said}");
        for stage in [Stage::Discovery, Stage::PlanReview, Stage::PlanAgreed, Stage::DirectionReview, Stage::Designing, Stage::DesignReview, Stage::Approved, Stage::ReadyToBuild, Stage::Building, Stage::Built] {
            assert!(!stage.line().is_empty());
        }
        for who in ["Rena", "Sora", "Koda"] {
            assert!(prompts::BUILD_FLOW_GUIDE.contains(who), "the order names {who}");
        }
    }
}
