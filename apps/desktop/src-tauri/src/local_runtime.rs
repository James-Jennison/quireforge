//! M63's only executable adapter: one CPU-only, in-process local attempt.
//!
//! The model location is supplied solely by the supervisor-owned environment
//! contract. It is never returned, persisted, logged, copied, or inspected by
//! this module beyond the read-only load performed by llama.cpp.

use serde::Serialize;
use std::{
    ffi::{c_char, c_void, CStr, CString},
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

const MODEL_PATH_ENV: &str = "QUIRE_FORGE_M63_MODEL_PATH";
const INPUT_TOKEN_LIMIT: usize = 4_096;
const OUTPUT_TOKEN_LIMIT: usize = 512;
const CONTEXT_TOKEN_LIMIT: usize = INPUT_TOKEN_LIMIT + OUTPUT_TOKEN_LIMIT;
const OUTPUT_BYTE_LIMIT: usize = 16 * 1024;
const CHAT_PROMPT_BYTE_LIMIT: usize = 2 * (96 * 1024 + 256);
const DEADLINE: Duration = Duration::from_secs(60);
const MEMORY_CEILING_BYTES: u64 = 6 * 1024 * 1024 * 1024;
const MEMORY_CEILING_MIB: u16 = 6 * 1024;
const REVIEW_SYSTEM_PROMPT: &str =
    "You are a local, offline assistant. Answer the reviewed request only.";
const LOCAL_CHAT_SYSTEM_PROMPT: &str =
    "You are a local, offline assistant. Answer the user's message directly and concisely.";

#[repr(C)]
struct ModelParams {
    devices: *mut c_void,
    tensor_buft_overrides: *const c_void,
    n_gpu_layers: i32,
    split_mode: i32,
    load_mode: i32,
    main_gpu: i32,
    tensor_split: *const f32,
    progress_callback: *const c_void,
    progress_callback_user_data: *mut c_void,
    kv_overrides: *const c_void,
    vocab_only: bool,
    check_tensors: bool,
    use_extra_bufts: bool,
    no_host: bool,
    no_alloc: bool,
    load_mtp: bool,
}

#[repr(C)]
struct ContextParams {
    n_ctx: u32,
    n_batch: u32,
    n_ubatch: u32,
    n_seq_max: u32,
    n_rs_seq: u32,
    n_outputs_max: u32,
    n_threads: i32,
    n_threads_batch: i32,
    ctx_type: i32,
    rope_scaling_type: i32,
    pooling_type: i32,
    attention_type: i32,
    flash_attn_type: i32,
    rope_freq_base: f32,
    rope_freq_scale: f32,
    yarn_ext_factor: f32,
    yarn_attn_factor: f32,
    yarn_beta_fast: f32,
    yarn_beta_slow: f32,
    yarn_orig_ctx: u32,
    defrag_thold: f32,
    cb_eval: *const c_void,
    cb_eval_user_data: *mut c_void,
    type_k: i32,
    type_v: i32,
    abort_callback: *const c_void,
    abort_callback_data: *mut c_void,
    embeddings: bool,
    offload_kqv: bool,
    no_perf: bool,
    op_offload: bool,
    swa_full: bool,
    kv_unified: bool,
    samplers: *mut c_void,
    n_samplers: usize,
    ctx_other: *mut c_void,
}

#[repr(C)]
struct Batch {
    n_tokens: i32,
    token: *mut i32,
    embd: *mut f32,
    pos: *mut i32,
    n_seq_id: *mut i32,
    seq_id: *mut *mut i32,
    logits: *mut i8,
}

#[repr(C)]
struct SamplerChainParams {
    no_perf: bool,
}

#[repr(C)]
struct ChatMessage {
    role: *const c_char,
    content: *const c_char,
}

unsafe extern "C" {
    fn llama_backend_init();
    fn llama_backend_free();
    fn llama_log_set(
        callback: unsafe extern "C" fn(i32, *const c_char, *mut c_void),
        user_data: *mut c_void,
    );
    fn llama_model_default_params() -> ModelParams;
    fn llama_context_default_params() -> ContextParams;
    fn llama_sampler_chain_default_params() -> SamplerChainParams;
    fn llama_model_load_from_file(path: *const c_char, params: ModelParams) -> *mut c_void;
    fn llama_model_free(model: *mut c_void);
    fn llama_init_from_model(model: *mut c_void, params: ContextParams) -> *mut c_void;
    fn llama_free(context: *mut c_void);
    fn llama_model_get_vocab(model: *const c_void) -> *const c_void;
    fn llama_model_chat_template(model: *const c_void, name: *const c_char) -> *const c_char;
    fn llama_chat_apply_template(
        template: *const c_char,
        chat: *const ChatMessage,
        message_count: usize,
        add_assistant: bool,
        buffer: *mut c_char,
        buffer_length: i32,
    ) -> i32;
    fn llama_tokenize(
        vocab: *const c_void,
        text: *const c_char,
        text_len: i32,
        tokens: *mut i32,
        n_tokens: i32,
        add_special: bool,
        parse_special: bool,
    ) -> i32;
    fn llama_batch_get_one(tokens: *mut i32, n_tokens: i32) -> Batch;
    fn llama_decode(context: *mut c_void, batch: Batch) -> i32;
    fn llama_sampler_chain_init(params: SamplerChainParams) -> *mut c_void;
    fn llama_sampler_chain_add(chain: *mut c_void, sampler: *mut c_void);
    fn llama_sampler_init_greedy() -> *mut c_void;
    fn llama_sampler_sample(sampler: *mut c_void, context: *mut c_void, index: i32) -> i32;
    fn llama_sampler_accept(sampler: *mut c_void, token: i32);
    fn llama_sampler_free(sampler: *mut c_void);
    fn llama_vocab_is_eog(vocab: *const c_void, token: i32) -> bool;
    fn llama_token_to_piece(
        vocab: *const c_void,
        token: i32,
        buffer: *mut c_char,
        length: i32,
        lstrip: i32,
        special: bool,
    ) -> i32;
}

struct Backend;

impl Backend {
    unsafe fn initialize() -> Self {
        llama_backend_init();
        Self
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        unsafe { llama_backend_free() }
    }
}

struct RunControl {
    started: Instant,
    cancelled: AtomicBool,
}

unsafe extern "C" fn continue_before_deadline(_: f32, data: *mut c_void) -> bool {
    !attempt_stopped(data)
}

// llama.cpp writes its default diagnostics to stderr, including model-load
// details. M63's model location is supervisor-owned and must not cross the
// local runtime boundary, so the adapter deliberately discards those details.
unsafe extern "C" fn discard_runtime_log(_: i32, _: *const c_char, _: *mut c_void) {}

struct LoaderProbe {
    category: AtomicU8,
}

fn loader_failure_category(bytes: &[u8]) -> u8 {
    // Match only a fixed, non-sensitive error class. The raw loader message is
    // intentionally neither retained nor returned because it can contain the
    // supervisor-owned model location.
    if bytes
        .windows(b"failed to open".len())
        .any(|part| part == b"failed to open")
    {
        1
    } else if bytes
        .windows(b"invalid".len())
        .any(|part| part == b"invalid")
        || bytes
            .windows(b"unsupported".len())
            .any(|part| part == b"unsupported")
    {
        2
    } else if bytes
        .windows(b"failed to allocate".len())
        .any(|part| part == b"failed to allocate")
        || bytes
            .windows(b"allocation failed".len())
            .any(|part| part == b"allocation failed")
        || bytes
            .windows(b"out of memory".len())
            .any(|part| part == b"out of memory")
        || bytes
            .windows(b"cannot allocate".len())
            .any(|part| part == b"cannot allocate")
    {
        3
    } else {
        0
    }
}

fn loader_failure_diagnostic(category: u8) -> &'static str {
    match category {
        1 => "model-access-failed",
        2 => "model-format-invalid",
        3 => "model-memory-unavailable",
        _ => "model-load-failed",
    }
}

unsafe extern "C" fn classify_loader_log(_: i32, text: *const c_char, data: *mut c_void) {
    let Some(probe) = data.cast::<LoaderProbe>().as_ref() else {
        return;
    };
    if text.is_null() {
        return;
    }
    let bytes = CStr::from_ptr(text).to_bytes();
    probe
        .category
        .fetch_max(loader_failure_category(bytes), Ordering::Relaxed);
}

unsafe extern "C" fn abort_at_deadline(data: *mut c_void) -> bool {
    attempt_stopped(data)
}

unsafe fn attempt_stopped(data: *mut c_void) -> bool {
    data.cast::<RunControl>().as_ref().is_none_or(|control| {
        control.cancelled.load(Ordering::Acquire) || control.started.elapsed() >= DEADLINE
    })
}

fn stopped(control: &RunControl) -> LocalRuntimeSnapshot {
    if control.cancelled.load(Ordering::Acquire) {
        cancelled()
    } else {
        failed("deadline-exceeded")
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LocalRuntimeSnapshot {
    pub schema_version: u16,
    pub local_only: bool,
    pub state: String,
    pub output: Option<String>,
    pub diagnostic: Option<String>,
    pub input_token_limit: u16,
    pub output_token_limit: u16,
    pub deadline_seconds: u8,
    pub memory_ceiling_mib: u16,
}

/// Content-free readiness for the supervisor-provided local model contract.
/// This deliberately reports neither the model location nor any filesystem
/// observation; it only prevents a one-use reviewed bundle from being
/// consumed when the required runtime input was not supplied.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LocalRuntimeAvailability {
    pub schema_version: u16,
    pub local_only: bool,
    pub available: bool,
    pub diagnostic: Option<String>,
}

#[derive(Default)]
pub(crate) struct LocalRuntimeService {
    active: Arc<Mutex<Option<ActiveRun>>>,
    // A successful content-free availability check retains the supervisor
    // contract only in this process until the next check or app exit. Binding
    // it to the reservation prevents an independently re-read environment
    // from invalidating the exact reviewed action between those two native
    // commands. It is never serialized, logged, or returned across IPC.
    model_contract: Arc<Mutex<Option<ModelContract>>>,
}

#[derive(Clone)]
struct ModelContract {
    model_path: String,
}

struct ActiveRunGuard {
    active: Arc<Mutex<Option<ActiveRun>>>,
}

struct ActiveRun {
    bundle_id: String,
    control: Arc<RunControl>,
}

/// An admission held for the exact bundle until its one local attempt returns.
/// Holding this before durable dispatch ensures a busy runtime cannot consume a
/// second reviewed bundle merely to reject it.
pub(crate) struct LocalRuntimeReservation {
    control: Arc<RunControl>,
    _active_run: ActiveRunGuard,
    model_contract: Option<ModelContract>,
}

/// M63 bounds resident memory with the supervisor service's cgroup, rather
/// than `RLIMIT_AS`: file-backed model mappings count toward address space but
/// are not a measure of memory actually committed by the runtime.
fn cgroup_memory_max_path(cgroup: &str) -> Option<PathBuf> {
    let relative = cgroup.strip_prefix('/')?;
    let path = Path::new(relative);
    if path
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        Some(Path::new("/sys/fs/cgroup").join(path).join("memory.max"))
    } else {
        None
    }
}

fn memory_ceiling_is_enforced() -> bool {
    let cgroup = std::fs::read_to_string("/proc/self/cgroup")
        .ok()
        .and_then(|contents| {
            contents
                .lines()
                .find_map(|line| line.strip_prefix("0::").map(str::to_owned))
        });
    cgroup
        .as_deref()
        .and_then(cgroup_memory_max_path)
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|value| value.trim().parse::<u64>().ok())
        .is_some_and(|limit| limit <= MEMORY_CEILING_BYTES)
}

impl Drop for ActiveRunGuard {
    fn drop(&mut self) {
        if let Ok(mut active) = self.active.lock() {
            active.take();
        }
    }
}

impl LocalRuntimeService {
    pub(crate) fn availability(&self) -> LocalRuntimeAvailability {
        let (contract, diagnostic) = match model_contract(std::env::var(MODEL_PATH_ENV)) {
            Some(contract) => match verify_model_load(&contract.model_path) {
                Ok(()) => (Some(contract), None),
                Err(diagnostic) => (None, Some(diagnostic)),
            },
            None => (None, Some("model-unavailable")),
        };
        let available = self
            .model_contract
            .lock()
            .map(|mut cached| {
                *cached = contract;
                cached.is_some()
            })
            .unwrap_or(false);
        LocalRuntimeAvailability {
            schema_version: 1,
            local_only: true,
            available,
            diagnostic: (!available).then(|| diagnostic.unwrap_or("model-unavailable").into()),
        }
    }

    /// Returns the bounded outcome used when the required supervisor-owned
    /// runtime input is unavailable. Callers must return this before reserving
    /// or consuming a reviewed bundle.
    pub(crate) fn unavailable_snapshot() -> LocalRuntimeSnapshot {
        failed("model-unavailable")
    }

    pub(crate) fn reserve(&self, bundle_id: &str) -> Result<LocalRuntimeReservation, ()> {
        let Ok(mut active) = self.active.lock() else {
            return Err(());
        };
        if active.is_some() {
            return Err(());
        }
        let control = Arc::new(RunControl {
            started: Instant::now(),
            cancelled: AtomicBool::new(false),
        });
        *active = Some(ActiveRun {
            bundle_id: bundle_id.into(),
            control: Arc::clone(&control),
        });
        Ok(LocalRuntimeReservation {
            control,
            _active_run: ActiveRunGuard {
                active: Arc::clone(&self.active),
            },
            model_contract: None,
        })
    }

    /// Binds the most recently verified supervisor contract to this exact
    /// reservation. The caller must invoke `availability` immediately before
    /// this method so native command handling remains authoritative even when
    /// invoked without the browser view.
    pub(crate) fn reserve_available_model(
        &self,
        bundle_id: &str,
    ) -> Result<LocalRuntimeReservation, ()> {
        let contract = self
            .model_contract
            .lock()
            .ok()
            .and_then(|cached| cached.clone())
            .ok_or(())?;
        let mut reservation = self.reserve(bundle_id)?;
        reservation.model_contract = Some(contract);
        Ok(reservation)
    }

    /// Reserves the same bounded CPU-only runtime for an M69A local-chat turn.
    /// The caller supplies no project, source, or reviewed-context identifier.
    pub(crate) fn reserve_local_chat(&self) -> Result<LocalRuntimeReservation, ()> {
        self.reserve_available_model("local-chat")
    }

    pub(crate) fn request_cancel(&self, bundle_id: &str) -> bool {
        let Ok(active) = self.active.lock() else {
            return false;
        };
        let Some(active) = active.as_ref() else {
            return false;
        };
        if active.bundle_id != bundle_id {
            return false;
        }
        active.control.cancelled.store(true, Ordering::Release);
        true
    }

    pub(crate) fn request_cancel_local_chat(&self) -> bool {
        self.request_cancel("local-chat")
    }
}

fn model_contract(model_path: Result<String, std::env::VarError>) -> Option<ModelContract> {
    model_path.ok().and_then(|model_path| {
        (!model_path.is_empty() && !model_path.as_bytes().contains(&0))
            .then_some(ModelContract { model_path })
    })
}

/// Performs the same CPU-only model admission used by an attempt, without
/// accepting reviewed bytes or returning any model-derived data. This closes
/// the gap where an environment string could pass preflight but native loading
/// would fail only after a one-use bundle had been consumed.
fn verify_model_load(model_path: &str) -> Result<(), &'static str> {
    let Ok(path) = CString::new(model_path) else {
        return Err("model-contract-invalid");
    };
    if !memory_ceiling_is_enforced() {
        return Err("memory-ceiling-unavailable");
    }
    let control = RunControl {
        started: Instant::now(),
        cancelled: AtomicBool::new(false),
    };
    unsafe {
        let probe = LoaderProbe {
            category: AtomicU8::new(0),
        };
        llama_log_set(
            classify_loader_log,
            (&probe as *const LoaderProbe).cast_mut().cast(),
        );
        let _backend = Backend::initialize();
        let mut model_params = llama_model_default_params();
        model_params.n_gpu_layers = 0;
        model_params.split_mode = 0;
        model_params.progress_callback = continue_before_deadline as *const c_void;
        model_params.progress_callback_user_data =
            (&control as *const RunControl).cast_mut().cast();
        let model = llama_model_load_from_file(path.as_ptr(), model_params);
        let result = if model.is_null() {
            Err(loader_failure_diagnostic(
                probe.category.load(Ordering::Relaxed),
            ))
        } else {
            llama_model_free(model);
            Ok(())
        };
        // llama.cpp owns a process-global callback. Restore the redacting
        // callback before `probe` goes out of scope so no future log can touch
        // a stale pointer.
        llama_log_set(discard_runtime_log, std::ptr::null_mut());
        result
    }
}

impl LocalRuntimeReservation {
    pub(crate) fn run(self, canonical_bytes: &[u8]) -> LocalRuntimeSnapshot {
        let Some(contract) = self.model_contract.as_ref() else {
            return LocalRuntimeService::unavailable_snapshot();
        };
        run_once(
            canonical_bytes,
            &self.control,
            &contract.model_path,
            REVIEW_SYSTEM_PROMPT,
            "reviewed-input-invalid",
            "reviewed-input-too-large",
        )
    }

    /// M69A's typed service owns validation and presents the resulting text
    /// only in the ephemeral chat view. This method deliberately accepts bytes
    /// alone, so no project/source/review authority can enter the runtime.
    pub(crate) fn run_local_chat(self, text: &[u8]) -> LocalRuntimeSnapshot {
        let Some(contract) = self.model_contract.as_ref() else {
            return LocalRuntimeService::unavailable_snapshot();
        };
        run_once(
            text,
            &self.control,
            &contract.model_path,
            &local_chat_system_prompt(),
            "local-chat-invalid",
            "local-chat-too-large",
        )
    }
}

fn local_chat_system_prompt() -> String {
    match local_clock() {
        Some(clock) => format!(
            "{LOCAL_CHAT_SYSTEM_PROMPT} The current local date and time is {clock}; use it for date or time questions. Never emit placeholder text or claim that you lack real-time capability."
        ),
        None => format!(
            "{LOCAL_CHAT_SYSTEM_PROMPT} Do not invent a date or emit placeholder text when the calendar is unavailable."
        ),
    }
}

pub(crate) fn local_clock() -> Option<String> {
    let mut now: libc::time_t = 0;
    let mut local: libc::tm = unsafe { std::mem::zeroed() };
    unsafe {
        if libc::time(&mut now).is_negative() || libc::localtime_r(&now, &mut local).is_null() {
            None
        } else {
            let mut formatted = [0_i8; 64];
            let written = libc::strftime(
                formatted.as_mut_ptr(),
                formatted.len(),
                c"%Y-%m-%d %H:%M %Z".as_ptr(),
                &local,
            );
            (written > 0).then(|| {
                CStr::from_ptr(formatted.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            })
        }
    }
}

fn run_once(
    canonical_bytes: &[u8],
    control: &RunControl,
    model_path: &str,
    system_prompt_text: &str,
    invalid_input_diagnostic: &'static str,
    oversized_input_diagnostic: &'static str,
) -> LocalRuntimeSnapshot {
    let Ok(path) = CString::new(model_path) else {
        return failed("runtime-unavailable");
    };
    if !memory_ceiling_is_enforced() {
        return failed("memory-ceiling-unavailable");
    }
    let user_message = match std::str::from_utf8(canonical_bytes) {
        Ok(value) => value,
        Err(_) => return failed(invalid_input_diagnostic),
    };
    let Ok(system_prompt) = CString::new(system_prompt_text) else {
        return failed("runtime-unavailable");
    };
    let Ok(user_message) = CString::new(user_message) else {
        return failed(invalid_input_diagnostic);
    };
    unsafe {
        llama_log_set(discard_runtime_log, std::ptr::null_mut());
        let _backend = Backend::initialize();
        let mut model_params = llama_model_default_params();
        model_params.n_gpu_layers = 0;
        model_params.split_mode = 0;
        model_params.progress_callback = continue_before_deadline as *const c_void;
        model_params.progress_callback_user_data = (control as *const RunControl).cast_mut().cast();
        let model = llama_model_load_from_file(path.as_ptr(), model_params);
        if model.is_null() {
            return if attempt_stopped((control as *const RunControl).cast_mut().cast()) {
                stopped(control)
            } else {
                failed("model-load-failed")
            };
        }
        let prompt = match format_chat_prompt(model, &system_prompt, &user_message) {
            Ok(prompt) => prompt,
            Err(diagnostic) => {
                llama_model_free(model);
                return if diagnostic == "chat-input-too-large" {
                    failed(oversized_input_diagnostic)
                } else {
                    failed(diagnostic)
                };
            }
        };
        let mut context_params = llama_context_default_params();
        // Preserve the fixed 4,096-token reviewed input allowance while
        // reserving the separately fixed 512-token output allowance.
        context_params.n_ctx = CONTEXT_TOKEN_LIMIT as u32;
        context_params.n_batch = 512;
        context_params.n_ubatch = 512;
        context_params.n_threads = 1;
        context_params.n_threads_batch = 1;
        context_params.offload_kqv = false;
        context_params.op_offload = false;
        context_params.no_perf = true;
        context_params.abort_callback = abort_at_deadline as *const c_void;
        context_params.abort_callback_data = (control as *const RunControl).cast_mut().cast();
        let context = llama_init_from_model(model, context_params);
        if context.is_null() {
            llama_model_free(model);
            return failed("context-init-failed");
        }
        let vocab = llama_model_get_vocab(model);
        let mut tokens = vec![0_i32; INPUT_TOKEN_LIMIT + 1];
        let token_count = llama_tokenize(
            vocab,
            prompt.as_ptr(),
            prompt.as_bytes().len() as i32,
            tokens.as_mut_ptr(),
            tokens.len() as i32,
            true,
            true,
        );
        if token_count <= 0 || token_count as usize > INPUT_TOKEN_LIMIT {
            llama_free(context);
            llama_model_free(model);
            return failed(oversized_input_diagnostic);
        }
        tokens.truncate(token_count as usize);
        if llama_decode(
            context,
            llama_batch_get_one(tokens.as_mut_ptr(), token_count),
        ) != 0
        {
            llama_free(context);
            llama_model_free(model);
            return if attempt_stopped((control as *const RunControl).cast_mut().cast()) {
                stopped(control)
            } else {
                failed("runtime-failed")
            };
        }
        let sampler = llama_sampler_chain_init(llama_sampler_chain_default_params());
        if sampler.is_null() {
            llama_free(context);
            llama_model_free(model);
            return failed("sampler-init-failed");
        }
        llama_sampler_chain_add(sampler, llama_sampler_init_greedy());
        let mut output = String::new();
        for _ in 0..OUTPUT_TOKEN_LIMIT {
            if attempt_stopped((control as *const RunControl).cast_mut().cast()) {
                llama_sampler_free(sampler);
                llama_free(context);
                llama_model_free(model);
                return stopped(control);
            }
            let token = llama_sampler_sample(sampler, context, -1);
            if llama_vocab_is_eog(vocab, token) {
                break;
            }
            let mut piece = [0_i8; 256];
            let piece_len = llama_token_to_piece(
                vocab,
                token,
                piece.as_mut_ptr(),
                piece.len() as i32,
                0,
                false,
            );
            if piece_len <= 0 || output.len().saturating_add(piece_len as usize) > OUTPUT_BYTE_LIMIT
            {
                break;
            }
            output.push_str(&String::from_utf8_lossy(std::slice::from_raw_parts(
                piece.as_ptr().cast::<u8>(),
                piece_len as usize,
            )));
            llama_sampler_accept(sampler, token);
            let mut next = [token];
            if llama_decode(context, llama_batch_get_one(next.as_mut_ptr(), 1)) != 0 {
                llama_sampler_free(sampler);
                llama_free(context);
                llama_model_free(model);
                return if attempt_stopped((control as *const RunControl).cast_mut().cast()) {
                    stopped(control)
                } else {
                    failed("runtime-failed")
                };
            }
        }
        llama_sampler_free(sampler);
        llama_free(context);
        llama_model_free(model);
        LocalRuntimeSnapshot {
            schema_version: 1,
            local_only: true,
            state: "completed".into(),
            output: Some(output),
            diagnostic: None,
            input_token_limit: INPUT_TOKEN_LIMIT as u16,
            output_token_limit: OUTPUT_TOKEN_LIMIT as u16,
            deadline_seconds: 60,
            memory_ceiling_mib: MEMORY_CEILING_MIB,
        }
    }
}

/// Formats the fixed local message through the model's embedded chat
/// template. The selected Qwen descriptor supplies that template; guessing a
/// prompt wire format here would make the adapter model-dependent in an
/// unreviewed way.
unsafe fn format_chat_prompt(
    model: *mut c_void,
    system_prompt: &CString,
    user_message: &CString,
) -> Result<CString, &'static str> {
    let template = llama_model_chat_template(model, std::ptr::null());
    if template.is_null() {
        return Err("chat-template-unavailable");
    }
    let system_role = c"system";
    let user_role = c"user";
    let messages = [
        ChatMessage {
            role: system_role.as_ptr(),
            content: system_prompt.as_ptr(),
        },
        ChatMessage {
            role: user_role.as_ptr(),
            content: user_message.as_ptr(),
        },
    ];
    let required = llama_chat_apply_template(
        template,
        messages.as_ptr(),
        messages.len(),
        true,
        std::ptr::null_mut(),
        0,
    );
    if required <= 0 || required as usize > CHAT_PROMPT_BYTE_LIMIT {
        return Err("chat-input-too-large");
    }
    let mut formatted = vec![0_i8; required as usize + 1];
    let written = llama_chat_apply_template(
        template,
        messages.as_ptr(),
        messages.len(),
        true,
        formatted.as_mut_ptr(),
        formatted.len() as i32,
    );
    if written != required {
        return Err("chat-template-unavailable");
    }
    CString::new(
        formatted[..written as usize]
            .iter()
            .map(|byte| *byte as u8)
            .collect::<Vec<_>>(),
    )
    .map_err(|_| "chat-template-unavailable")
}

fn failed(diagnostic: &str) -> LocalRuntimeSnapshot {
    LocalRuntimeSnapshot {
        schema_version: 1,
        local_only: true,
        state: "failed".into(),
        output: None,
        diagnostic: Some(diagnostic.into()),
        input_token_limit: INPUT_TOKEN_LIMIT as u16,
        output_token_limit: OUTPUT_TOKEN_LIMIT as u16,
        deadline_seconds: 60,
        memory_ceiling_mib: MEMORY_CEILING_MIB,
    }
}

fn cancelled() -> LocalRuntimeSnapshot {
    LocalRuntimeSnapshot {
        schema_version: 1,
        local_only: true,
        state: "cancelled".into(),
        output: None,
        diagnostic: Some("cancelled".into()),
        input_token_limit: INPUT_TOKEN_LIMIT as u16,
        output_token_limit: OUTPUT_TOKEN_LIMIT as u16,
        deadline_seconds: 60,
        memory_ceiling_mib: MEMORY_CEILING_MIB,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cancelled, cgroup_memory_max_path, loader_failure_category, loader_failure_diagnostic,
        local_chat_system_prompt, memory_ceiling_is_enforced, model_contract, LocalRuntimeService,
        CHAT_PROMPT_BYTE_LIMIT, LOCAL_CHAT_SYSTEM_PROMPT, MEMORY_CEILING_BYTES, MEMORY_CEILING_MIB,
        REVIEW_SYSTEM_PROMPT,
    };

    #[test]
    fn cancellation_has_a_distinct_terminal_snapshot() {
        let snapshot = cancelled();
        assert_eq!(snapshot.state, "cancelled");
        assert_eq!(snapshot.diagnostic.as_deref(), Some("cancelled"));
    }

    #[test]
    fn local_runtime_memory_ceiling_matches_the_approved_six_gib_limit() {
        assert_eq!(MEMORY_CEILING_BYTES, 6 * 1024 * 1024 * 1024);
        assert_eq!(MEMORY_CEILING_MIB, 6 * 1024);
    }

    #[test]
    fn memory_ceiling_requires_a_finite_supervisor_cgroup_limit() {
        // Host test environments need not be launched by the M63 supervisor;
        // this assertion only proves the checker is total and content-free.
        let _ = memory_ceiling_is_enforced();
    }

    #[test]
    fn memory_ceiling_uses_only_the_process_cgroup_memory_file() {
        assert_eq!(
            cgroup_memory_max_path("/user.slice/user-1000.slice/app.slice/quireforge.service")
                .as_deref(),
            Some(std::path::Path::new(
                "/sys/fs/cgroup/user.slice/user-1000.slice/app.slice/quireforge.service/memory.max"
            ))
        );
        assert!(cgroup_memory_max_path("relative").is_none());
        assert!(cgroup_memory_max_path("/user.slice/../escape").is_none());
    }

    #[test]
    fn local_runtime_uses_a_bounded_two_message_chat_prompt() {
        assert!(REVIEW_SYSTEM_PROMPT.contains("reviewed request"));
        assert!(LOCAL_CHAT_SYSTEM_PROMPT.contains("user's message"));
        assert_eq!(CHAT_PROMPT_BYTE_LIMIT, 2 * (96 * 1024 + 256));
    }

    #[test]
    fn local_chat_prompt_has_no_date_placeholder() {
        let prompt = local_chat_system_prompt();
        assert!(!prompt.contains("[insert current date]"));
        assert!(
            prompt.contains("Never emit placeholder text")
                || prompt.contains("Do not invent a date")
        );
        assert!(prompt.contains("current local date and time"));
    }

    #[test]
    fn availability_is_content_free_when_the_model_contract_is_missing() {
        assert!(model_contract(Err(std::env::VarError::NotPresent)).is_none());
        let unavailable = LocalRuntimeService::unavailable_snapshot();
        assert_eq!(unavailable.state, "failed");
        assert_eq!(unavailable.diagnostic.as_deref(), Some("model-unavailable"));
    }

    #[test]
    fn loader_failures_are_reduced_to_fixed_content_free_categories() {
        assert_eq!(
            loader_failure_diagnostic(loader_failure_category(b"failed to open model")),
            "model-access-failed"
        );
        assert_eq!(
            loader_failure_diagnostic(loader_failure_category(b"invalid model header")),
            "model-format-invalid"
        );
        assert_eq!(
            loader_failure_diagnostic(loader_failure_category(b"allocation failed")),
            "model-memory-unavailable"
        );
        assert_eq!(
            loader_failure_category(b"model memory footprint: bounded"),
            0
        );
        assert_eq!(
            loader_failure_diagnostic(loader_failure_category(b"unclassified loader failure")),
            "model-load-failed"
        );
    }

    #[test]
    fn local_runtime_rejects_a_second_concurrent_attempt_and_releases_afterward() {
        let runtime = LocalRuntimeService::default();
        let active = runtime
            .reserve("bundle-a")
            .expect("first attempt claims the slot");
        assert!(
            runtime.reserve("bundle-b").is_err(),
            "second attempt is rejected"
        );
        assert!(
            runtime.request_cancel("bundle-a"),
            "exact bundle can cancel"
        );
        assert!(
            !runtime.request_cancel("bundle-b"),
            "other bundles cannot cancel the active attempt"
        );
        drop(active);
        assert!(
            runtime.reserve("bundle-c").is_ok(),
            "terminal attempt releases the slot"
        );
    }

    #[test]
    fn availability_binds_a_verified_contract_to_the_exact_reservation() {
        let runtime = LocalRuntimeService::default();
        {
            let mut cached = runtime.model_contract.lock().expect("contract lock");
            *cached = model_contract(Ok("supervisor-contract".into()));
        }
        let reservation = runtime
            .reserve_available_model("bundle-a")
            .expect("verified contract reserves the only slot");
        assert!(reservation.model_contract.is_some());
    }

    /// This is deliberately opt-in: it loads the supervisor-provided,
    /// read-only model through the same in-process adapter used by the Tauri
    /// command. It records no model location or generated output in the test
    /// result, while giving the local candidate a repeatable real-adapter
    /// acceptance gate before the separately required installed-host review.
    #[test]
    #[ignore = "requires the supervisor-provided M63 local model"]
    fn approved_model_completes_one_bounded_local_attempt() {
        let runtime = LocalRuntimeService::default();
        assert!(
            runtime.availability().available,
            "the approved local model contract must be available"
        );
        let reservation = runtime
            .reserve_available_model("m63-focused-adapter-acceptance")
            .expect("the focused attempt claims the only runtime slot");
        let snapshot = reservation.run(b"Provide a concise offline readiness confirmation.");

        assert_eq!(snapshot.state, "completed");
        assert!(snapshot.local_only);
        assert_eq!(snapshot.diagnostic, None);
        assert!(snapshot
            .output
            .is_some_and(|output| output.len() <= 16 * 1024));
        assert_eq!(snapshot.input_token_limit, 4096);
        assert_eq!(snapshot.output_token_limit, 512);
        assert_eq!(snapshot.deadline_seconds, 60);
        assert_eq!(snapshot.memory_ceiling_mib, 6144);
    }
}
