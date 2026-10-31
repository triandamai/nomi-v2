//! SQL for each table. Reads take a connection; writes take a [`Tx`](super::database::Tx) so they
//! join the caller's transaction and mark their table changed.

pub(crate) mod activity;
pub(crate) mod drafts;
pub(crate) mod messages;
pub(crate) mod notifications;
pub(crate) mod outbox;
pub(crate) mod sessions;
pub(crate) mod sync_state;

/// The largest page any query returns, whatever the caller asks for.
pub(crate) const MAX_PAGE_SIZE: u32 = 500;

pub(crate) fn clamp_limit(limit: u32) -> u32 {
    limit.clamp(1, MAX_PAGE_SIZE)
}
