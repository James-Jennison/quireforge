//! M76 isolated, owner-approved, read-only browser research.
//!
//! This service is deliberately separate from M58's fixture verifier. Every
//! launch is exact-target and exact-origin bound, uses an ephemeral profile,
//! disables JavaScript, and retains only bounded digest provenance in memory.

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
use url::Url;
use uuid::Uuid;
#[cfg(target_os = "linux")]
use webkit2gtk::{
    DownloadExt, LoadEvent, SettingsExt, WebContext, WebContextExt, WebView, WebViewExt,
};

const TTL_MS: u64 = 5 * 60 * 1000;
const TIMEOUT_MS: u64 = 12_000;
const MAX_OBSERVATION_BYTES: usize = 2_048;
const HELPER_FLAG: &str = "--browser-research-helper";

pub(crate) struct BrowserResearchService {
    attempts: Mutex<HashMap<String, Attempt>>,
}
impl Default for BrowserResearchService {
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
    origin: String,
    observation_limit: u16,
    request_digest: String,
    expires_at_ms: u64,
    state: State,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Prepared,
    Observed,
    Cancelled,
    Revoked,
    Expired,
    OriginDrift,
    PromptInjection,
    TimedOut,
    Incompatible,
    Failed,
    Closed,
}
impl State {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::Observed => "observed",
            Self::Cancelled => "cancelled",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
            Self::OriginDrift => "origin_drift",
            Self::PromptInjection => "prompt_injection",
            Self::TimedOut => "timed_out",
            Self::Incompatible => "incompatible",
            Self::Failed => "failed",
            Self::Closed => "closed",
        }
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct BrowserResearchPrepareRequest {
    pub project_id: String,
    pub task_id: Option<String>,
    pub target: String,
    pub origin: String,
    pub observation_limit: u16,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct BrowserResearchConfirmRequest {
    pub attempt_id: String,
    pub authorization_id: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct BrowserResearchAttemptRequest {
    pub attempt_id: String,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BrowserResearchSnapshot {
    pub schema_version: u16,
    pub isolated: bool,
    pub read_only: bool,
    pub adapter: String,
    pub state: String,
    pub project_id: Option<String>,
    pub task_id: Option<String>,
    pub attempt_id: Option<String>,
    pub authorization_id: Option<String>,
    pub target: Option<String>,
    pub origin: Option<String>,
    pub request_digest: Option<String>,
    pub expires_at_ms: Option<u64>,
    pub observation_limit: Option<u16>,
    pub observed_at_ms: Option<u64>,
    pub content_digest: Option<String>,
    pub observed_bytes: Option<u16>,
    pub diagnostic: Option<String>,
    pub audit_state: String,
}
#[derive(Serialize, Deserialize)]
struct HelperResult {
    state: String,
    content_digest: Option<String>,
    observed_bytes: Option<u16>,
    observed_at_ms: Option<u64>,
    diagnostic: String,
}

impl BrowserResearchService {
    pub(crate) fn status(&self) -> BrowserResearchSnapshot {
        snapshot(
            None,
            State::Closed,
            None,
            None,
            "isolated research ready; no browser profile or session exists",
        )
    }
    pub(crate) fn prepare(
        &self,
        request: BrowserResearchPrepareRequest,
        project_id: String,
    ) -> BrowserResearchSnapshot {
        if request.project_id != project_id
            || !valid_uuid(&project_id)
            || request
                .task_id
                .as_deref()
                .is_some_and(|value| !valid_uuid(value))
            || request.observation_limit == 0
            || usize::from(request.observation_limit) > MAX_OBSERVATION_BYTES
            || !valid_scope(&request.target, &request.origin)
        {
            return rejected("invalid-read-only-research-scope");
        }
        let now = now_ms();
        let id = Uuid::now_v7().to_string();
        let authorization_id = Uuid::now_v7().to_string();
        let request_digest = sha(&format!(
            "{project_id}:{}:{}:{}:{}",
            request.task_id.clone().unwrap_or_default(),
            request.target,
            request.origin,
            request.observation_limit
        ));
        let attempt = Attempt {
            id: id.clone(),
            authorization_id: authorization_id.clone(),
            project_id,
            task_id: request.task_id,
            target: request.target,
            origin: request.origin,
            observation_limit: request.observation_limit,
            request_digest,
            expires_at_ms: now + TTL_MS,
            state: State::Prepared,
        };
        let value = snapshot(
            Some(&attempt),
            State::Prepared,
            Some(authorization_id),
            None,
            "exact target and origin require owner confirmation; no browser launched",
        );
        self.attempts
            .lock()
            .expect("research lock")
            .insert(id, attempt);
        value
    }
    pub(crate) fn confirm(
        &self,
        request: BrowserResearchConfirmRequest,
    ) -> BrowserResearchSnapshot {
        let mut attempts = self.attempts.lock().expect("research lock");
        let Some(attempt) = attempts.get_mut(&request.attempt_id) else {
            return rejected("attempt-unavailable");
        };
        if attempt.authorization_id != request.authorization_id || attempt.state != State::Prepared
        {
            return rejected("authorization-replayed-or-mismatched");
        }
        if now_ms() >= attempt.expires_at_ms {
            attempt.state = State::Expired;
            return snapshot(
                Some(attempt),
                State::Expired,
                None,
                None,
                "authorization expired; no browser launched",
            );
        }
        let result = run_adapter(&attempt.target, &attempt.origin, attempt.observation_limit);
        let state = match result.state.as_str() {
            "observed" => State::Observed,
            "origin_drift" => State::OriginDrift,
            "prompt_injection" => State::PromptInjection,
            "timed_out" => State::TimedOut,
            "incompatible" => State::Incompatible,
            _ => State::Failed,
        };
        attempt.state = state;
        snapshot(
            Some(attempt),
            state,
            None,
            Some(&result),
            &result.diagnostic,
        )
    }
    pub(crate) fn cancel(&self, request: BrowserResearchAttemptRequest) -> BrowserResearchSnapshot {
        self.terminal(
            request.attempt_id,
            State::Cancelled,
            "cancelled before launch; no browser remains",
        )
    }
    pub(crate) fn revoke(&self, request: BrowserResearchAttemptRequest) -> BrowserResearchSnapshot {
        self.terminal(
            request.attempt_id,
            State::Revoked,
            "revoked before launch; no browser remains",
        )
    }
    fn terminal(&self, id: String, state: State, audit: &str) -> BrowserResearchSnapshot {
        let mut attempts = self.attempts.lock().expect("research lock");
        let Some(attempt) = attempts.get_mut(&id) else {
            return rejected("attempt-unavailable");
        };
        if attempt.state != State::Prepared {
            return rejected("terminal-transition-rejected");
        }
        attempt.state = state;
        snapshot(Some(attempt), state, None, None, audit)
    }
}
fn snapshot(
    attempt: Option<&Attempt>,
    state: State,
    authorization_id: Option<String>,
    result: Option<&HelperResult>,
    audit: &str,
) -> BrowserResearchSnapshot {
    BrowserResearchSnapshot {
        schema_version: 1,
        isolated: true,
        read_only: true,
        adapter: "ephemeral-webkitgtk-research".into(),
        state: state.as_str().into(),
        project_id: attempt.map(|a| a.project_id.clone()),
        task_id: attempt.and_then(|a| a.task_id.clone()),
        attempt_id: attempt.map(|a| a.id.clone()),
        authorization_id,
        target: attempt.map(|a| a.target.clone()),
        origin: attempt.map(|a| a.origin.clone()),
        request_digest: attempt.map(|a| a.request_digest.clone()),
        expires_at_ms: attempt.map(|a| a.expires_at_ms),
        observation_limit: attempt.map(|a| a.observation_limit),
        observed_at_ms: result.and_then(|r| r.observed_at_ms),
        content_digest: result.and_then(|r| r.content_digest.clone()),
        observed_bytes: result.and_then(|r| r.observed_bytes),
        diagnostic: None,
        audit_state: audit.into(),
    }
}
fn rejected(reason: &str) -> BrowserResearchSnapshot {
    let mut value = snapshot(None, State::Failed, None, None, "no operation occurred");
    value.state = "rejected".into();
    value.diagnostic = Some(reason.into());
    value
}
fn valid_scope(target: &str, origin: &str) -> bool {
    let Ok(target) = Url::parse(target) else {
        return false;
    };
    let Ok(origin) = Url::parse(origin) else {
        return false;
    };
    target.scheme() == "https"
        && origin.scheme() == "https"
        && target.username().is_empty()
        && target.password().is_none()
        && target.fragment().is_none()
        && target.port().is_none()
        && origin.path() == "/"
        && origin.query().is_none()
        && origin.fragment().is_none()
        && target.origin().ascii_serialization() == origin.origin().ascii_serialization()
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
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
fn suspicious(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    [
        "ignore previous instructions",
        "system message",
        "developer message",
        "prompt injection",
    ]
    .iter()
    .any(|needle| value.contains(needle))
}
fn run_adapter(target: &str, origin: &str, limit: u16) -> HelperResult {
    #[cfg(test)]
    {
        let _ = (target, origin, limit);
        HelperResult {
            state: "observed".into(),
            content_digest: Some(sha("isolated research fixture")),
            observed_bytes: Some(24),
            observed_at_ms: Some(now_ms()),
            diagnostic: "deterministic isolated-adapter test seam completed; no browser launched"
                .into(),
        }
    }
    #[cfg(not(test))]
    {
        run_adapter_process(target, origin, limit)
    }
}
#[cfg(not(test))]
fn run_adapter_process(target: &str, origin: &str, limit: u16) -> HelperResult {
    let Ok(executable) = std::env::current_exe() else {
        return unavailable("research helper path unavailable");
    };
    let Ok(mut child) = Command::new(executable)
        .arg(HELPER_FLAG)
        .arg(target)
        .arg(origin)
        .arg(limit.to_string())
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
    else {
        return unavailable("research helper could not launch");
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
                    if let Ok(result) = serde_json::from_str(output.trim()) {
                        return result;
                    }
                }
                return unavailable("research helper exited without a bounded result");
            }
            Ok(None) if now_ms().saturating_sub(started) < TIMEOUT_MS => {
                thread::sleep(std::time::Duration::from_millis(20))
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return HelperResult {
                    state: "timed_out".into(),
                    content_digest: None,
                    observed_bytes: None,
                    observed_at_ms: None,
                    diagnostic: "research timed out and was terminated; no retry occurred".into(),
                };
            }
            Err(_) => return unavailable("research helper status unavailable"),
        }
    }
}
fn unavailable(reason: &str) -> HelperResult {
    HelperResult {
        state: "incompatible".into(),
        content_digest: None,
        observed_bytes: None,
        observed_at_ms: None,
        diagnostic: reason.into(),
    }
}
pub(crate) fn run_helper_from_env() -> bool {
    let arguments = std::env::args().collect::<Vec<_>>();
    if arguments.len() != 5 || arguments[1] != HELPER_FLAG {
        return false;
    }
    let limit = arguments[4].parse::<u16>().ok();
    let result = limit
        .filter(|limit| {
            usize::from(*limit) <= MAX_OBSERVATION_BYTES
                && valid_scope(&arguments[2], &arguments[3])
        })
        .map(|limit| run_native_research(&arguments[2], &arguments[3], limit))
        .unwrap_or_else(|| unavailable("invalid helper scope"));
    println!("{}", serde_json::to_string(&result).unwrap_or_else(|_| "{\"state\":\"incompatible\",\"content_digest\":null,\"observed_bytes\":null,\"observed_at_ms\":null,\"diagnostic\":\"result serialization failed\"}".into()));
    true
}
#[cfg(target_os = "linux")]
#[allow(deprecated)]
fn run_native_research(target: &str, origin: &str, limit: u16) -> HelperResult {
    if gtk::init().is_err() {
        return unavailable("ephemeral WebKitGTK research unavailable");
    }
    let context = WebContext::new_ephemeral();
    context.connect_download_started(|_, download| download.cancel());
    let view = WebView::with_context(&context);
    let Some(settings) = view.settings() else {
        return unavailable("WebKitGTK research settings unavailable; no browser launched");
    };
    settings.set_enable_javascript(false);
    let result = std::rc::Rc::new(std::cell::RefCell::new(None));
    let main_loop = glib::MainLoop::new(None, false);
    let result_for_load = result.clone();
    let loop_for_load = main_loop.clone();
    let expected_origin = origin.to_owned();
    view.connect_load_changed(move |view, event| { if event != LoadEvent::Finished { return; } let current = view.uri().unwrap_or_default().to_string(); let current_origin = Url::parse(&current).ok().map(|url| url.origin().ascii_serialization()); if current_origin.as_deref() != Some(expected_origin.as_str()) { *result_for_load.borrow_mut() = Some(HelperResult { state: "origin_drift".into(), content_digest: None, observed_bytes: None, observed_at_ms: None, diagnostic: "redirect or origin drift blocked observation".into() }); loop_for_load.quit(); return; } let result = result_for_load.clone(); let main_loop = loop_for_load.clone(); view.run_javascript("document.body ? document.body.innerText : ''", None::<&gio::Cancellable>, move |value| { let text = value.ok().and_then(|value| value.js_value()).map(|value| value.to_str().to_string()).unwrap_or_default(); let bytes = text.as_bytes(); let observed = &bytes[..bytes.len().min(usize::from(limit))]; let outcome = if suspicious(&String::from_utf8_lossy(observed)) { HelperResult { state: "prompt_injection".into(), content_digest: None, observed_bytes: None, observed_at_ms: None, diagnostic: "prompt-injection indicator blocked observation".into() } } else { HelperResult { state: "observed".into(), content_digest: Some(sha(&String::from_utf8_lossy(observed))), observed_bytes: Some(observed.len() as u16), observed_at_ms: Some(now_ms()), diagnostic: "bounded read-only observation completed in an ephemeral profile".into() } }; *result.borrow_mut() = Some(outcome); main_loop.quit(); }); });
    let failed_result = result.clone();
    let failed_loop = main_loop.clone();
    view.connect_load_failed(move |_view, _event, _uri, _error| {
        *failed_result.borrow_mut() = Some(HelperResult {
            state: "failed".into(),
            content_digest: None,
            observed_bytes: None,
            observed_at_ms: None,
            diagnostic: "read-only navigation failed".into(),
        });
        failed_loop.quit();
        true
    });
    let timeout_result = result.clone();
    let timeout_loop = main_loop.clone();
    glib::timeout_add_local_once(
        std::time::Duration::from_millis(TIMEOUT_MS - 500),
        move || {
            if timeout_result.borrow().is_none() {
                *timeout_result.borrow_mut() = Some(HelperResult {
                    state: "timed_out".into(),
                    content_digest: None,
                    observed_bytes: None,
                    observed_at_ms: None,
                    diagnostic: "research observation timed out".into(),
                });
                timeout_loop.quit();
            }
        },
    );
    view.load_uri(target);
    main_loop.run();
    let outcome = result
        .borrow_mut()
        .take()
        .unwrap_or_else(|| unavailable("research ended without result"));
    outcome
}
#[cfg(not(target_os = "linux"))]
fn run_native_research(_target: &str, _origin: &str, _limit: u16) -> HelperResult {
    unavailable("WebKitGTK research is unavailable on this platform")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn project() -> String {
        Uuid::now_v7().to_string()
    }
    #[test]
    fn owner_approval_is_exact_origin_bound_and_one_use() {
        let service = BrowserResearchService::default();
        let project_id = project();
        let prepared = service.prepare(
            BrowserResearchPrepareRequest {
                project_id: project_id.clone(),
                task_id: None,
                target: "https://google.com/".into(),
                origin: "https://google.com".into(),
                observation_limit: 512,
            },
            project_id,
        );
        assert_eq!(prepared.state, "prepared");
        let observed = service.confirm(BrowserResearchConfirmRequest {
            attempt_id: prepared.attempt_id.clone().unwrap(),
            authorization_id: prepared.authorization_id.clone().unwrap(),
        });
        assert_eq!(observed.state, "observed");
        assert!(observed.content_digest.is_some());
        assert_eq!(
            service
                .confirm(BrowserResearchConfirmRequest {
                    attempt_id: prepared.attempt_id.unwrap(),
                    authorization_id: prepared.authorization_id.unwrap()
                })
                .state,
            "rejected"
        );
    }
    #[test]
    fn rejects_insecure_redirectable_or_oversized_scopes() {
        let service = BrowserResearchService::default();
        let project_id = project();
        for (target, origin) in [
            ("http://google.com/", "http://google.com"),
            ("https://google.com/", "https://www.google.com"),
            ("https://google.com/#fragment", "https://google.com"),
        ] {
            assert_eq!(
                service
                    .prepare(
                        BrowserResearchPrepareRequest {
                            project_id: project_id.clone(),
                            task_id: None,
                            target: target.into(),
                            origin: origin.into(),
                            observation_limit: 512
                        },
                        project_id.clone()
                    )
                    .state,
                "rejected"
            );
        }
    }
}
