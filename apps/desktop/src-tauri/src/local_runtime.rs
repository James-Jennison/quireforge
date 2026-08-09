//! M63's only executable adapter: one CPU-only, in-process local attempt.
//!
//! The model location is supplied solely by the supervisor-owned environment
//! contract. It is never returned, persisted, logged, copied, or inspected by
//! this module beyond the read-only load performed by llama.cpp.

use serde::Serialize;
use std::{
    ffi::{c_char, c_void, CString},
    sync::{
        atomic::{AtomicBool, Ordering},
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
const MEMORY_CEILING_BYTES: libc::rlim_t = 6 * 1024 * 1024 * 1024;
const SYSTEM_PROMPT: &str = "You are a local, offline assistant. Answer the reviewed request only.";

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
}

#[derive(Default)]
pub(crate) struct LocalRuntimeService {
    active: Mutex<Option<ActiveRun>>,
}

struct ActiveRunGuard<'a> {
    active: &'a Mutex<Option<ActiveRun>>,
}

struct ActiveRun {
    bundle_id: String,
    control: Arc<RunControl>,
}

/// Temporarily confines model allocation to M63's fixed address-space budget.
/// The previous soft limit is restored as the local attempt exits.
struct MemoryCeiling {
    previous: Option<libc::rlimit>,
}

impl MemoryCeiling {
    fn apply() -> Result<Self, ()> {
        let mut previous = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: reads this process's resource limit without retaining pointers.
        if unsafe { libc::getrlimit(libc::RLIMIT_AS, &mut previous) } != 0 {
            return Err(());
        }
        if previous.rlim_cur <= MEMORY_CEILING_BYTES {
            return Ok(Self { previous: None });
        }
        let constrained = libc::rlimit {
            rlim_cur: MEMORY_CEILING_BYTES.min(previous.rlim_max),
            rlim_max: previous.rlim_max,
        };
        // SAFETY: reduces only the soft limit for this bounded attempt; Drop
        // restores the exact observed value before the command returns.
        if unsafe { libc::setrlimit(libc::RLIMIT_AS, &constrained) } != 0 {
            return Err(());
        }
        Ok(Self {
            previous: Some(previous),
        })
    }
}

impl Drop for MemoryCeiling {
    fn drop(&mut self) {
        if let Some(previous) = self.previous {
            // SAFETY: restores the exact soft/hard values observed by apply.
            let _ = unsafe { libc::setrlimit(libc::RLIMIT_AS, &previous) };
        }
    }
}

impl Drop for ActiveRunGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut active) = self.active.lock() {
            active.take();
        }
    }
}

impl LocalRuntimeService {
    pub(crate) fn run(&self, bundle_id: &str, canonical_bytes: &[u8]) -> LocalRuntimeSnapshot {
        let Ok((control, _active_run)) = self.claim_slot(bundle_id) else {
            return failed("runtime-busy");
        };
        run_once(canonical_bytes, &control)
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

    fn claim_slot(&self, bundle_id: &str) -> Result<(Arc<RunControl>, ActiveRunGuard<'_>), ()> {
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
        Ok((
            control,
            ActiveRunGuard {
                active: &self.active,
            },
        ))
    }
}

fn run_once(canonical_bytes: &[u8], control: &RunControl) -> LocalRuntimeSnapshot {
    let Ok(model_path) = std::env::var(MODEL_PATH_ENV) else {
        return failed("model-unavailable");
    };
    if model_path.is_empty() || model_path.as_bytes().contains(&0) {
        return failed("model-unavailable");
    }
    let Ok(path) = CString::new(model_path) else {
        return failed("model-unavailable");
    };
    let Ok(_memory_ceiling) = MemoryCeiling::apply() else {
        return failed("memory-ceiling-unavailable");
    };
    let reviewed_request = match std::str::from_utf8(canonical_bytes) {
        Ok(value) => value,
        Err(_) => return failed("reviewed-input-invalid"),
    };
    let Ok(system_prompt) = CString::new(SYSTEM_PROMPT) else {
        return failed("runtime-unavailable");
    };
    let Ok(reviewed_request) = CString::new(reviewed_request) else {
        return failed("reviewed-input-invalid");
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
                failed("model-unavailable")
            };
        }
        let prompt = match format_reviewed_chat_prompt(model, &system_prompt, &reviewed_request) {
            Ok(prompt) => prompt,
            Err(diagnostic) => {
                llama_model_free(model);
                return failed(diagnostic);
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
            return failed("runtime-unavailable");
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
            return failed("reviewed-input-too-large");
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
            return failed("runtime-unavailable");
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
        }
    }
}

/// Formats the fixed reviewed request through the model's embedded chat
/// template. The selected Qwen descriptor supplies that template; guessing a
/// prompt wire format here would make the adapter model-dependent in an
/// unreviewed way.
unsafe fn format_reviewed_chat_prompt(
    model: *mut c_void,
    system_prompt: &CString,
    reviewed_request: &CString,
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
            content: reviewed_request.as_ptr(),
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
        return Err("reviewed-input-too-large");
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
    .map_err(|_| "reviewed-input-invalid")
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
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cancelled, LocalRuntimeService, CHAT_PROMPT_BYTE_LIMIT, MEMORY_CEILING_BYTES, SYSTEM_PROMPT,
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
    }

    #[test]
    fn local_runtime_uses_a_bounded_two_message_chat_prompt() {
        assert!(SYSTEM_PROMPT.contains("local, offline assistant"));
        assert_eq!(CHAT_PROMPT_BYTE_LIMIT, 2 * (96 * 1024 + 256));
    }

    #[test]
    fn local_runtime_rejects_a_second_concurrent_attempt_and_releases_afterward() {
        let runtime = LocalRuntimeService::default();
        let (_, active) = runtime
            .claim_slot("bundle-a")
            .expect("first attempt claims the slot");
        assert!(
            runtime.claim_slot("bundle-b").is_err(),
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
            runtime.claim_slot("bundle-c").is_ok(),
            "terminal attempt releases the slot"
        );
    }
}
