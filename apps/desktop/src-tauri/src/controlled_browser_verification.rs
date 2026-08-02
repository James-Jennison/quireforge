//! M58 fictional, local-only, read-only controlled browser verification.
//!
//! This controller owns no network route, profile, credential, connector, MCP,
//! provider, native-tool, or mutation authority. Its fixture URI is served only
//! by a fresh ephemeral WebKitGTK context when the later adapter is launched.
#[cfg(target_os = "linux")]
use gtk::{gio, glib};
#[cfg(target_os = "linux")]
use javascriptcore::ValueExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
#[cfg(not(test))]
use std::{
    io::Read,
    process::{Command, Stdio},
    thread,
};
use uuid::Uuid;
#[cfg(target_os = "linux")]
use webkit2gtk::{
    LoadEvent, SecurityManagerExt, URISchemeRequestExt, WebContext, WebContextExt, WebView,
    WebViewExt,
};

const TTL_MS: u64 = 5 * 60 * 1000;
const FIXTURE_ORIGIN: &str = "quireforge-fixture://verification";
const FIXTURE_URL: &str = "quireforge-fixture://verification/expected?assert=marker";
const FIXTURE_HELPER_FLAG: &str = "--controlled-browser-fixture-helper";
const FIXTURE_TIMEOUT_MS: u64 = 8_000;
const FIXTURE_HTML: &str = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>QuireForge fictional fixture marker</title></head><body><main aria-label=\"Fictional local-only verification fixture\">fixture-marker</main></body></html>";

#[derive(Serialize, Deserialize)]
struct FixtureHelperResult {
    state: String,
    evidence_digest: Option<String>,
    visible_text: Option<String>,
    diagnostic: String,
}

pub(crate) struct ControlledBrowserVerificationService {
    attempts: Mutex<HashMap<String, Attempt>>,
}

impl Default for ControlledBrowserVerificationService {
    fn default() -> Self {
        Self {
            attempts: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Clone)]
struct Attempt {
    id: String,
    authorization_id: String,
    project_id: String,
    task_id: Option<String>,
    target: String,
    assertion: String,
    digest: String,
    expires_at_ms: u64,
    state: BrowserState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum BrowserState {
    Prepared,
    Confirmed,
    Verified,
    VerificationFailed,
    Cancelled,
    Denied,
    Expired,
    Revoked,
    RedirectBlocked,
    OriginDrift,
    TimedOut,
    Ambiguous,
    Quarantined,
    Incompatible,
    Closed,
}

impl BrowserState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::Confirmed => "confirmed",
            Self::Verified => "verified",
            Self::VerificationFailed => "verification_failed",
            Self::Cancelled => "cancelled",
            Self::Denied => "denied",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
            Self::RedirectBlocked => "redirect_blocked",
            Self::OriginDrift => "origin_drift",
            Self::TimedOut => "timed_out",
            Self::Ambiguous => "ambiguous",
            Self::Quarantined => "quarantined",
            Self::Incompatible => "incompatible",
            Self::Closed => "closed",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct BrowserVerificationPrepareRequest {
    pub project_id: String,
    pub task_id: Option<String>,
    pub target: String,
    pub assertion: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct BrowserVerificationConfirmRequest {
    pub attempt_id: String,
    pub authorization_id: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct BrowserVerificationAttemptRequest {
    pub attempt_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BrowserVerificationSnapshot {
    pub schema_version: u16,
    pub fictional_local_only: bool,
    pub read_only: bool,
    pub adapter: String,
    pub state: String,
    pub project_id: Option<String>,
    pub task_id: Option<String>,
    pub attempt_id: Option<String>,
    pub authorization_id: Option<String>,
    pub target: Option<String>,
    pub origin: Option<String>,
    pub assertion: Option<String>,
    pub request_digest: Option<String>,
    pub expires_at_ms: Option<u64>,
    pub evidence_digest: Option<String>,
    pub visible_text: Option<String>,
    pub diagnostic: Option<String>,
    pub audit_state: String,
}

impl ControlledBrowserVerificationService {
    pub(crate) fn status(&self) -> BrowserVerificationSnapshot {
        snapshot(
            None,
            BrowserState::Closed,
            None,
            "fictional local-only verifier ready; no session, network, or browser process exists",
        )
    }

    pub(crate) fn prepare(
        &self,
        request: BrowserVerificationPrepareRequest,
        project_id: String,
    ) -> BrowserVerificationSnapshot {
        if request.project_id != project_id
            || !valid_uuid(&project_id)
            || request
                .task_id
                .as_deref()
                .is_some_and(|value| !valid_uuid(value))
            || request.target != FIXTURE_URL
            || request.assertion != "fixture-marker"
        {
            return rejected("invalid-local-fixture-request");
        }
        let now = now_ms();
        let id = Uuid::now_v7().to_string();
        let authorization_id = Uuid::now_v7().to_string();
        let digest = sha(&format!(
            "{project_id}:{}:{}:{}",
            request.task_id.clone().unwrap_or_default(),
            request.target,
            request.assertion
        ));
        let attempt = Attempt {
            id: id.clone(),
            authorization_id: authorization_id.clone(),
            project_id,
            task_id: request.task_id,
            target: request.target,
            assertion: request.assertion,
            digest,
            expires_at_ms: now + TTL_MS,
            state: BrowserState::Prepared,
        };
        let result = snapshot(
            Some(&attempt),
            BrowserState::Prepared,
            Some(authorization_id),
            "review required; preparation launched no adapter or fixture",
        );
        self.attempts
            .lock()
            .expect("browser verifier lock")
            .insert(id, attempt);
        result
    }

    pub(crate) fn confirm(
        &self,
        request: BrowserVerificationConfirmRequest,
    ) -> BrowserVerificationSnapshot {
        let mut attempts = self.attempts.lock().expect("browser verifier lock");
        let Some(attempt) = attempts.get_mut(&request.attempt_id) else {
            return rejected("attempt-unavailable");
        };
        if attempt.authorization_id != request.authorization_id
            || attempt.state != BrowserState::Prepared
        {
            return rejected("authorization-replayed-or-mismatched");
        }
        if now_ms() >= attempt.expires_at_ms {
            attempt.state = BrowserState::Expired;
            return snapshot(
                Some(attempt),
                attempt.state,
                None,
                "authorization expired; no adapter launched",
            );
        }
        attempt.state = BrowserState::Confirmed;
        // The helper is a separately spawned, short-lived process. Its only
        // load is a native-served custom-scheme document; it receives neither
        // a profile path nor a caller-provided URL.
        let result = run_fixture_adapter();
        let state = match result.state.as_str() {
            "verified" => BrowserState::Verified,
            "timed_out" => BrowserState::TimedOut,
            "ambiguous" => BrowserState::Ambiguous,
            "incompatible" => BrowserState::Incompatible,
            _ => BrowserState::VerificationFailed,
        };
        attempt.state = state;
        let mut value = snapshot(Some(attempt), state, None, &result.diagnostic);
        if let (BrowserState::Verified, Some(evidence), Some(visible_text)) =
            (state, result.evidence_digest, result.visible_text)
        {
            value = value.with_evidence(evidence, &visible_text);
        }
        value
    }

    pub(crate) fn cancel(
        &self,
        request: BrowserVerificationAttemptRequest,
    ) -> BrowserVerificationSnapshot {
        self.terminal(
            request.attempt_id,
            BrowserState::Cancelled,
            "cancelled before verification; no adapter remains",
        )
    }
    pub(crate) fn revoke(
        &self,
        request: BrowserVerificationAttemptRequest,
    ) -> BrowserVerificationSnapshot {
        self.terminal(
            request.attempt_id,
            BrowserState::Revoked,
            "revoked; no adapter remains",
        )
    }
    #[allow(dead_code)]
    pub(crate) fn fixture_outcome(
        &self,
        request: BrowserVerificationAttemptRequest,
        outcome: &str,
    ) -> BrowserVerificationSnapshot {
        let state = match outcome {
            "redirect" => BrowserState::RedirectBlocked,
            "drift" => BrowserState::OriginDrift,
            "timeout" => BrowserState::TimedOut,
            "ambiguous" => BrowserState::Ambiguous,
            "incompatible" => BrowserState::Incompatible,
            "quarantined" => BrowserState::Quarantined,
            _ => BrowserState::VerificationFailed,
        };
        self.terminal(
            request.attempt_id,
            state,
            "fictional fixture terminal outcome; automatic retry is prohibited",
        )
    }
    fn terminal(
        &self,
        id: String,
        state: BrowserState,
        audit: &str,
    ) -> BrowserVerificationSnapshot {
        let mut attempts = self.attempts.lock().expect("browser verifier lock");
        let Some(attempt) = attempts.get_mut(&id) else {
            return rejected("attempt-unavailable");
        };
        if attempt.state != BrowserState::Prepared {
            return rejected("terminal-transition-rejected");
        }
        attempt.state = state;
        snapshot(Some(attempt), state, None, audit)
    }
}

impl BrowserVerificationSnapshot {
    fn with_evidence(mut self, evidence_digest: String, visible_text: &str) -> Self {
        self.evidence_digest = Some(evidence_digest);
        self.visible_text = Some(visible_text.into());
        self
    }
}

fn snapshot(
    attempt: Option<&Attempt>,
    state: BrowserState,
    authorization: Option<String>,
    audit: &str,
) -> BrowserVerificationSnapshot {
    BrowserVerificationSnapshot {
        schema_version: 1,
        fictional_local_only: true,
        read_only: true,
        adapter: "ephemeral-webkitgtk-fixture".into(),
        state: state.as_str().into(),
        project_id: attempt.map(|a| a.project_id.clone()),
        task_id: attempt.and_then(|a| a.task_id.clone()),
        attempt_id: attempt.map(|a| a.id.clone()),
        authorization_id: authorization,
        target: attempt.map(|a| a.target.clone()),
        origin: attempt.map(|_| FIXTURE_ORIGIN.into()),
        assertion: attempt.map(|a| a.assertion.clone()),
        request_digest: attempt.map(|a| a.digest.clone()),
        expires_at_ms: attempt.map(|a| a.expires_at_ms),
        evidence_digest: None,
        visible_text: None,
        diagnostic: None,
        audit_state: audit.into(),
    }
}
fn rejected(reason: &str) -> BrowserVerificationSnapshot {
    let mut value = snapshot(None, BrowserState::Closed, None, "no operation occurred");
    value.state = "rejected".into();
    value.diagnostic = Some(reason.into());
    value
}
fn valid_uuid(value: &str) -> bool {
    Uuid::parse_str(value).is_ok()
}
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn sha(value: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(value.as_bytes());
    format!("{:x}", hash.finalize())
}

fn run_fixture_adapter() -> FixtureHelperResult {
    #[cfg(test)]
    {
        FixtureHelperResult {
            state: "verified".into(),
            evidence_digest: Some(sha("fixture-marker:QuireForge fictional fixture marker")),
            visible_text: Some("fixture marker verified".into()),
            diagnostic: "deterministic fixture-adapter test seam completed; no process launched"
                .into(),
        }
    }
    #[cfg(not(test))]
    run_fixture_adapter_process()
}

#[cfg(not(test))]
fn run_fixture_adapter_process() -> FixtureHelperResult {
    let executable = match std::env::current_exe() {
        Ok(value) => value,
        Err(_) => {
            return FixtureHelperResult {
                state: "ambiguous".into(),
                evidence_digest: None,
                visible_text: None,
                diagnostic: "fixture helper path unavailable; no verification occurred".into(),
            }
        }
    };
    let mut child = match Command::new(executable)
        .arg(FIXTURE_HELPER_FLAG)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(value) => value,
        Err(_) => {
            return FixtureHelperResult {
                state: "incompatible".into(),
                evidence_digest: None,
                visible_text: None,
                diagnostic: "fixture helper could not launch; no verification occurred".into(),
            }
        }
    };
    let started = now_ms();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut output = String::new();
                if let Some(mut stdout) = child.stdout.take() {
                    let _ = stdout.read_to_string(&mut output);
                }
                if status.success() {
                    if let Ok(value) = serde_json::from_str::<FixtureHelperResult>(output.trim()) {
                        return value;
                    }
                }
                return FixtureHelperResult {
                    state: "ambiguous".into(),
                    evidence_digest: None,
                    visible_text: None,
                    diagnostic:
                        "fixture helper exited without complete evidence; no retry occurred".into(),
                };
            }
            Ok(None) if now_ms().saturating_sub(started) < FIXTURE_TIMEOUT_MS => {
                thread::sleep(std::time::Duration::from_millis(20));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return FixtureHelperResult {
                    state: "timed_out".into(),
                    evidence_digest: None,
                    visible_text: None,
                    diagnostic: "fixture helper timed out and was terminated; no retry occurred"
                        .into(),
                };
            }
            Err(_) => {
                return FixtureHelperResult {
                    state: "ambiguous".into(),
                    evidence_digest: None,
                    visible_text: None,
                    diagnostic: "fixture helper status became unavailable; no retry occurred"
                        .into(),
                }
            }
        }
    }
}

/// Runs only the fixed M58 fixture helper, before Tauri creates an application
/// window. The helper accepts no target, profile, credential, or navigation
/// argument and always exits after one local observation.
pub(crate) fn run_fixture_helper_from_env() -> bool {
    let arguments = std::env::args().collect::<Vec<_>>();
    if arguments.len() != 2 || arguments[1] != FIXTURE_HELPER_FLAG {
        return false;
    }
    let result = run_native_fixture();
    println!("{}", serde_json::to_string(&result).unwrap_or_else(|_| "{\"state\":\"ambiguous\",\"evidence_digest\":null,\"visible_text\":null,\"diagnostic\":\"fixture result serialization failed\"}".into()));
    true
}

#[cfg(target_os = "linux")]
#[allow(deprecated)] // WebKitGTK 4.1 exposes this bounded observation API.
fn run_native_fixture() -> FixtureHelperResult {
    if gtk::init().is_err() {
        return FixtureHelperResult {
            state: "incompatible".into(),
            evidence_digest: None,
            visible_text: None,
            diagnostic: "ephemeral WebKitGTK fixture unavailable; no verification occurred".into(),
        };
    }
    let context = WebContext::new_ephemeral();
    let Some(security_manager) = context.security_manager() else {
        return FixtureHelperResult {
            state: "incompatible".into(),
            evidence_digest: None,
            visible_text: None,
            diagnostic: "ephemeral WebKitGTK security policy unavailable; no verification occurred"
                .into(),
        };
    };
    security_manager.register_uri_scheme_as_local("quireforge-fixture");
    context.register_uri_scheme("quireforge-fixture", |request| {
        if request.uri().as_deref() == Some(FIXTURE_URL) {
            let bytes = glib::Bytes::from_static(FIXTURE_HTML.as_bytes());
            let stream = gio::MemoryInputStream::from_bytes(&bytes);
            request.finish(
                &stream,
                FIXTURE_HTML.len() as i64,
                Some("text/html; charset=utf-8"),
            );
        } else {
            let bytes = glib::Bytes::from_static(b"blocked");
            let stream = gio::MemoryInputStream::from_bytes(&bytes);
            request.finish(&stream, 7, Some("text/plain; charset=utf-8"));
        }
    });
    let view = WebView::with_context(&context);
    let result = std::rc::Rc::new(std::cell::RefCell::new(None));
    let main_loop = glib::MainLoop::new(None, false);
    let finish_loop = main_loop.clone();
    let finish_result = result.clone();
    view.connect_load_changed(move |view, event| {
        if event != LoadEvent::Finished {
            return;
        }
        let target = view.uri().map(|value| value.to_string());
        let result = finish_result.clone();
        let main_loop = finish_loop.clone();
        // This is a fixed read-only observation of the fixture's visible body
        // text. It cannot be supplied by a caller and cannot interact with a
        // page, navigate, or invoke any native command.
        view.run_javascript(
            "document.body ? document.body.innerText : ''",
            None::<&gio::Cancellable>,
            move |value| {
            let visible_text = value
                .ok()
                .and_then(|value| value.js_value())
                .map(|value| value.to_str().to_string());
            let outcome = if target.as_deref() == Some(FIXTURE_URL)
                && visible_text.as_deref() == Some("fixture-marker")
            {
                FixtureHelperResult {
                    state: "verified".into(),
                    evidence_digest: Some(sha("fixture-marker:QuireForge fictional fixture marker")),
                    visible_text: Some("fixture marker verified".into()),
                    diagnostic: "ephemeral WebKitGTK fixture completed and cleaned up; no network, session, interaction, or external effect".into(),
                }
            } else {
                FixtureHelperResult {
                    state: "verification_failed".into(), evidence_digest: None, visible_text: None,
                    diagnostic: "fixture evidence did not match the exact reviewed target".into(),
                }
            };
            *result.borrow_mut() = Some(outcome);
            main_loop.quit();
            },
        );
    });
    let failed_loop = main_loop.clone();
    let failed_result = result.clone();
    view.connect_load_failed(move |_view, _event, _uri, _error| {
        *failed_result.borrow_mut() = Some(FixtureHelperResult {
            state: "verification_failed".into(),
            evidence_digest: None,
            visible_text: None,
            diagnostic: "fixture navigation failed; no verification occurred".into(),
        });
        failed_loop.quit();
        true
    });
    let timeout_loop = main_loop.clone();
    let timeout_result = result.clone();
    glib::timeout_add_local_once(
        std::time::Duration::from_millis(FIXTURE_TIMEOUT_MS - 500),
        move || {
            if timeout_result.borrow().is_none() {
                *timeout_result.borrow_mut() = Some(FixtureHelperResult {
                    state: "timed_out".into(),
                    evidence_digest: None,
                    visible_text: None,
                    diagnostic: "fixture observation timed out; no retry occurred".into(),
                });
                timeout_loop.quit();
            }
        },
    );
    view.load_uri(FIXTURE_URL);
    main_loop.run();
    let final_result = result.borrow_mut().take().unwrap_or(FixtureHelperResult {
        state: "ambiguous".into(),
        evidence_digest: None,
        visible_text: None,
        diagnostic: "fixture exited without complete evidence; no retry occurred".into(),
    });
    final_result
}

#[cfg(not(target_os = "linux"))]
fn run_native_fixture() -> FixtureHelperResult {
    FixtureHelperResult {
        state: "incompatible".into(),
        evidence_digest: None,
        visible_text: None,
        diagnostic: "WebKitGTK fixture is unavailable on this platform".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn project() -> String {
        Uuid::now_v7().to_string()
    }
    #[test]
    fn prepare_is_digest_bound_and_does_not_launch() {
        let service = ControlledBrowserVerificationService::default();
        let project_id = project();
        let value = service.prepare(
            BrowserVerificationPrepareRequest {
                project_id: project_id.clone(),
                task_id: None,
                target: FIXTURE_URL.into(),
                assertion: "fixture-marker".into(),
            },
            project_id,
        );
        assert_eq!(value.state, "prepared");
        assert!(value.request_digest.is_some());
        assert!(value.evidence_digest.is_none());
        assert!(value.audit_state.contains("no adapter"));
    }
    #[test]
    fn confirmation_is_one_use_and_local_only() {
        let service = ControlledBrowserVerificationService::default();
        let project_id = project();
        let prepared = service.prepare(
            BrowserVerificationPrepareRequest {
                project_id: project_id.clone(),
                task_id: None,
                target: FIXTURE_URL.into(),
                assertion: "fixture-marker".into(),
            },
            project_id,
        );
        let verified = service.confirm(BrowserVerificationConfirmRequest {
            attempt_id: prepared.attempt_id.clone().unwrap(),
            authorization_id: prepared.authorization_id.clone().unwrap(),
        });
        assert_eq!(verified.state, "verified");
        assert!(verified.evidence_digest.is_some());
        let replay = service.confirm(BrowserVerificationConfirmRequest {
            attempt_id: prepared.attempt_id.unwrap(),
            authorization_id: prepared.authorization_id.unwrap(),
        });
        assert_eq!(replay.state, "rejected");
    }
    #[test]
    fn rejects_non_fixture_and_ambiguous_does_not_retry() {
        let service = ControlledBrowserVerificationService::default();
        let project_id = project();
        assert_eq!(
            service
                .prepare(
                    BrowserVerificationPrepareRequest {
                        project_id: project_id.clone(),
                        task_id: None,
                        target: "https://example.invalid".into(),
                        assertion: "fixture-marker".into()
                    },
                    project_id
                )
                .state,
            "rejected"
        );
        let project_id = project();
        let prepared = service.prepare(
            BrowserVerificationPrepareRequest {
                project_id: project_id.clone(),
                task_id: None,
                target: FIXTURE_URL.into(),
                assertion: "fixture-marker".into(),
            },
            project_id,
        );
        assert_eq!(
            service
                .fixture_outcome(
                    BrowserVerificationAttemptRequest {
                        attempt_id: prepared.attempt_id.clone().unwrap()
                    },
                    "ambiguous"
                )
                .state,
            "ambiguous"
        );
        assert_eq!(
            service
                .confirm(BrowserVerificationConfirmRequest {
                    attempt_id: prepared.attempt_id.unwrap(),
                    authorization_id: prepared.authorization_id.unwrap()
                })
                .state,
            "rejected"
        );
    }

    #[test]
    fn cancellation_revocation_and_expiry_are_terminal_without_adapter_execution() {
        let service = ControlledBrowserVerificationService::default();
        let project_id = project();
        let prepared = service.prepare(
            BrowserVerificationPrepareRequest {
                project_id: project_id.clone(),
                task_id: None,
                target: FIXTURE_URL.into(),
                assertion: "fixture-marker".into(),
            },
            project_id,
        );
        let attempt_id = prepared.attempt_id.clone().expect("attempt");
        assert_eq!(
            service
                .cancel(BrowserVerificationAttemptRequest {
                    attempt_id: attempt_id.clone()
                })
                .state,
            "cancelled"
        );
        assert_eq!(
            service
                .confirm(BrowserVerificationConfirmRequest {
                    attempt_id,
                    authorization_id: prepared.authorization_id.expect("authorization")
                })
                .state,
            "rejected"
        );

        let project_id = project();
        let prepared = service.prepare(
            BrowserVerificationPrepareRequest {
                project_id: project_id.clone(),
                task_id: None,
                target: FIXTURE_URL.into(),
                assertion: "fixture-marker".into(),
            },
            project_id,
        );
        assert_eq!(
            service
                .revoke(BrowserVerificationAttemptRequest {
                    attempt_id: prepared.attempt_id.expect("attempt")
                })
                .state,
            "revoked"
        );

        let project_id = project();
        let prepared = service.prepare(
            BrowserVerificationPrepareRequest {
                project_id: project_id.clone(),
                task_id: None,
                target: FIXTURE_URL.into(),
                assertion: "fixture-marker".into(),
            },
            project_id,
        );
        let attempt_id = prepared.attempt_id.clone().expect("attempt");
        service
            .attempts
            .lock()
            .expect("lock")
            .get_mut(&attempt_id)
            .expect("attempt")
            .expires_at_ms = 0;
        assert_eq!(
            service
                .confirm(BrowserVerificationConfirmRequest {
                    attempt_id,
                    authorization_id: prepared.authorization_id.expect("authorization")
                })
                .state,
            "expired"
        );
    }

    #[test]
    fn deterministic_fixture_failure_states_are_terminal_and_never_retry() {
        for outcome in [
            "redirect",
            "drift",
            "timeout",
            "incompatible",
            "quarantined",
        ] {
            let service = ControlledBrowserVerificationService::default();
            let project_id = project();
            let prepared = service.prepare(
                BrowserVerificationPrepareRequest {
                    project_id: project_id.clone(),
                    task_id: None,
                    target: FIXTURE_URL.into(),
                    assertion: "fixture-marker".into(),
                },
                project_id,
            );
            let attempt_id = prepared.attempt_id.clone().expect("attempt");
            let terminal = service.fixture_outcome(
                BrowserVerificationAttemptRequest {
                    attempt_id: attempt_id.clone(),
                },
                outcome,
            );
            assert_ne!(terminal.state, "prepared");
            assert_eq!(
                service
                    .confirm(BrowserVerificationConfirmRequest {
                        attempt_id,
                        authorization_id: prepared.authorization_id.expect("authorization")
                    })
                    .state,
                "rejected"
            );
        }
    }
}
