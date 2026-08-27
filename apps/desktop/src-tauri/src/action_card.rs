//! M69C's non-executing native Action Card authority boundary.
//!
//! Cards carry only a closed action class and opaque identifiers. They cannot
//! carry a project, path, source, prompt, provider, tool, or execution input.
//! Approval creates a content-free process-local receipt; it never performs an
//! action. A later capability-specific native service must explicitly consume a
//! compatible receipt before any boundary can be crossed.

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const MAX_CARDS: usize = 32;
const CARD_TTL_MILLIS: i64 = 5 * 60 * 1000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ActionCardAction {
    AttachProject,
    UseSource,
    DraftArtifact,
    WorkWithCode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ActionCardState {
    Prepared,
    Approved,
    Revoked,
    Expired,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ActionCardPrepareRequest {
    pub action: ActionCardAction,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ActionCardDecisionRequest {
    pub card_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActionCardSnapshot {
    pub schema_version: u8,
    pub card_id: String,
    pub action: ActionCardAction,
    pub state: ActionCardState,
    pub data_scope: &'static str,
    pub execution: &'static str,
    pub receipt_id: Option<String>,
    pub expires_at_ms: i64,
}

#[derive(Clone, Debug)]
struct StoredActionCard {
    id: Uuid,
    action: ActionCardAction,
    state: ActionCardState,
    receipt_id: Option<Uuid>,
    expires_at_ms: i64,
}

#[derive(Default)]
pub(crate) struct ActionCardService {
    cards: Mutex<HashMap<Uuid, StoredActionCard>>,
}

impl ActionCardService {
    pub(crate) fn prepare(
        &self,
        request: ActionCardPrepareRequest,
    ) -> Result<ActionCardSnapshot, ()> {
        let now = now_ms()?;
        let mut cards = self.cards.lock().map_err(|_| ())?;
        expire_cards(&mut cards, now);
        if cards.len() >= MAX_CARDS {
            return Err(());
        }
        let card = StoredActionCard {
            id: Uuid::now_v7(),
            action: request.action,
            state: ActionCardState::Prepared,
            receipt_id: None,
            expires_at_ms: now.checked_add(CARD_TTL_MILLIS).ok_or(())?,
        };
        let snapshot = snapshot(&card);
        cards.insert(card.id, card);
        Ok(snapshot)
    }

    pub(crate) fn approve(
        &self,
        request: ActionCardDecisionRequest,
    ) -> Result<ActionCardSnapshot, ()> {
        self.decide(request, true)
    }

    pub(crate) fn revoke(
        &self,
        request: ActionCardDecisionRequest,
    ) -> Result<ActionCardSnapshot, ()> {
        self.decide(request, false)
    }

    fn decide(
        &self,
        request: ActionCardDecisionRequest,
        approve: bool,
    ) -> Result<ActionCardSnapshot, ()> {
        let id = Uuid::parse_str(&request.card_id).map_err(|_| ())?;
        let now = now_ms()?;
        let mut cards = self.cards.lock().map_err(|_| ())?;
        expire_cards(&mut cards, now);
        let card = cards.get_mut(&id).ok_or(())?;
        if card.state != ActionCardState::Prepared {
            return Err(());
        }
        if approve {
            card.state = ActionCardState::Approved;
            card.receipt_id = Some(Uuid::now_v7());
        } else {
            card.state = ActionCardState::Revoked;
        }
        Ok(snapshot(card))
    }
}

fn expire_cards(cards: &mut HashMap<Uuid, StoredActionCard>, now: i64) {
    for card in cards.values_mut() {
        if card.state == ActionCardState::Prepared && card.expires_at_ms <= now {
            card.state = ActionCardState::Expired;
        }
    }
}

fn snapshot(card: &StoredActionCard) -> ActionCardSnapshot {
    ActionCardSnapshot {
        schema_version: 1,
        card_id: card.id.to_string(),
        action: card.action,
        state: card.state,
        data_scope: "none",
        execution: "not-authorized",
        receipt_id: card.receipt_id.map(|id| id.to_string()),
        expires_at_ms: card.expires_at_ms,
    }
}

fn now_ms() -> Result<i64, ()> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ())?
        .as_millis()
        .try_into()
        .map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::{
        ActionCardAction, ActionCardDecisionRequest, ActionCardPrepareRequest, ActionCardService,
        ActionCardState,
    };

    #[test]
    fn a_closed_card_approves_only_one_opaque_non_executing_receipt() {
        let service = ActionCardService::default();
        let prepared = service
            .prepare(ActionCardPrepareRequest {
                action: ActionCardAction::AttachProject,
            })
            .expect("prepared card");
        assert_eq!(prepared.state, ActionCardState::Prepared);
        assert_eq!(prepared.data_scope, "none");
        assert_eq!(prepared.execution, "not-authorized");
        assert!(prepared.receipt_id.is_none());

        let approved = service
            .approve(ActionCardDecisionRequest {
                card_id: prepared.card_id,
            })
            .expect("approved card");
        assert_eq!(approved.state, ActionCardState::Approved);
        assert_eq!(approved.data_scope, "none");
        assert_eq!(approved.execution, "not-authorized");
        assert!(approved.receipt_id.is_some());
        assert!(service
            .approve(ActionCardDecisionRequest {
                card_id: approved.card_id,
            })
            .is_err());
    }

    #[test]
    fn revocation_is_terminal_and_never_creates_a_receipt() {
        let service = ActionCardService::default();
        let prepared = service
            .prepare(ActionCardPrepareRequest {
                action: ActionCardAction::UseSource,
            })
            .expect("prepared card");
        let revoked = service
            .revoke(ActionCardDecisionRequest {
                card_id: prepared.card_id,
            })
            .expect("revoked card");
        assert_eq!(revoked.state, ActionCardState::Revoked);
        assert!(revoked.receipt_id.is_none());
        assert!(service
            .approve(ActionCardDecisionRequest {
                card_id: revoked.card_id,
            })
            .is_err());
    }

    #[test]
    fn request_types_reject_unknown_fields_and_cannot_carry_capability_data() {
        let prepare = serde_json::from_str::<ActionCardPrepareRequest>(
            r#"{"action":"work-with-code","path":"/not-allowed"}"#,
        );
        let decision = serde_json::from_str::<ActionCardDecisionRequest>(
            r#"{"cardId":"018f0000-0000-7000-8000-000000000000","prompt":"no"}"#,
        );
        assert!(prepare.is_err());
        assert!(decision.is_err());
    }
}
