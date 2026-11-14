//! Renews paid plans. Every hour: a plan whose paid period is about to end gets an invoice for
//! the same plan and term at today's prices (when it renews by itself) or one reminder (when it
//! doesn't), and a plan whose period and grace have run out gets one "your plan ended" notice.
//! `subscription_for` already puts such a person back on the default plan.

use std::time::Duration;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use nomi_usage::billing::{self, STATUS_PENDING};
use nomi_usage::plans;

use crate::billing::{self as server_billing, CheckoutError};

const SECONDS_PER_HOUR: u64 = 3600;
/// How often the worker looks.
const INTERVAL: Duration = Duration::from_secs(SECONDS_PER_HOUR);

/// What one pass did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub invoiced: usize,
    pub reminded: usize,
    pub ended: usize,
}

pub async fn run(pool: PgPool) {
    tracing::info!("billing worker: started");
    loop {
        match run_once(&pool, Utc::now()).await {
            Ok(summary) if summary != Summary::default() => tracing::info!(?summary, "billing worker: pass done"),
            Ok(_) => {}
            Err(e) => tracing::error!(error = %e, "billing worker: pass failed"),
        }
        tokio::time::sleep(INTERVAL).await;
    }
}

/// One pass at `now`.
pub async fn run_once(pool: &PgPool, now: DateTime<Utc>) -> Result<Summary, sqlx::Error> {
    let cfg = billing::config().await;
    let gateway = server_billing::gateway().await;
    let mut summary = Summary::default();

    for candidate in billing::renewal_candidates(pool, now, &cfg).await? {
        if candidate.auto_renew {
            let Some(gateway) = gateway.as_ref() else { continue };
            if billing::pending_renewal(pool, candidate.user_id).await?.is_some() {
                continue;
            }
            let quote = match billing::quote(pool, candidate.user_id, candidate.plan_id, candidate.term_months, None, &cfg).await {
                Ok(quote) => quote,
                Err(e) => {
                    tracing::warn!(error = %e, user_id = %candidate.user_id, "billing worker: can't price a renewal");
                    continue;
                }
            };
            match server_billing::start_payment(pool, candidate.user_id, &quote, billing::KIND_RENEWAL, &cfg, Some(gateway)).await {
                Ok(payment) => {
                    summary.invoiced += 1;
                    // Free renewals are activated on the spot and announced as paid.
                    if payment.status == STATUS_PENDING {
                        let locale = server_billing::user_locale(pool, candidate.user_id).await;
                        server_billing::tell(pool, candidate.user_id, server_billing::renewal_notice(locale, &candidate.plan_name, &payment, candidate.paid_until)).await;
                    }
                }
                Err(CheckoutError::Db(e)) => return Err(e),
                Err(e) => tracing::warn!(error = ?e, user_id = %candidate.user_id, "billing worker: couldn't invoice a renewal"),
            }
        } else if billing::claim_notice(pool, candidate.user_id, billing::NOTICE_REMINDER, candidate.paid_until).await? {
            let locale = server_billing::user_locale(pool, candidate.user_id).await;
            server_billing::tell(pool, candidate.user_id, server_billing::reminder_notice(locale, &candidate.plan_name, candidate.paid_until)).await;
            summary.reminded += 1;
        }
    }

    let lapsed = billing::lapsed_unannounced(pool, now, &cfg).await?;
    if !lapsed.is_empty() {
        let default_plan = plans::default_plan(pool).await?;
        for lapse in lapsed {
            if billing::claim_notice(pool, lapse.user_id, billing::NOTICE_ENDED, lapse.paid_until).await? {
                let locale = server_billing::user_locale(pool, lapse.user_id).await;
                server_billing::tell(pool, lapse.user_id, server_billing::ended_notice(locale, &lapse.plan_name, &default_plan.name, lapse.paid_until)).await;
                summary.ended += 1;
            }
        }
    }
    Ok(summary)
}
