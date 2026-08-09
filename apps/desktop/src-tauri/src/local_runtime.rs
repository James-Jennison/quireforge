//! M63's only executable adapter: one CPU-only, in-process local attempt.
//!
//! The model location is supplied solely by the supervisor-owned environment
//! contract. It is never returned, persisted, logged, copied, or inspected by
//! this module beyond the read-only load performed by llama.cpp.

use serde::Serialize;
use std::{
    ffi::{c_char, c_void, CString},
    ptr,
    sync::Mutex,
    time::{Duration, Instant},
};

const MODEL_PATH_ENV: &str = "QUIRE_FORGE_M63_MODEL_PATH";
const INPUT_TOKEN_LIMIT: usize = 4_096;
const OUTPUT_TOKEN_LIMIT: usize = 512;
const OUTPUT_BYTE_LIMIT: usize = 16 * 1024;
const DEADLINE: Duration = Duration::from_secs(60);

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

unsafe extern "C" {
    fn llama_backend_init();
    fn llama_model_default_params() -> ModelParams;
    fn llama_context_default_params() -> ContextParams;
    fn llama_sampler_chain_default_params() -> SamplerChainParams;
    fn llama_model_load_from_file(path: *const c_char, params: ModelParams) -> *mut c_void;
    fn llama_model_free(model: *mut c_void);
    fn llama_init_from_model(model: *mut c_void, params: ContextParams) -> *mut c_void;
    fn llama_free(context: *mut c_void);
    fn llama_model_get_vocab(model: *const c_void) -> *const c_void;
    fn llama_tokenize(vocab: *const c_void, text: *const c_char, text_len: i32, tokens: *mut i32, n_tokens: i32, add_special: bool, parse_special: bool) -> i32;
    fn llama_batch_get_one(tokens: *mut i32, n_tokens: i32) -> Batch;
    fn llama_decode(context: *mut c_void, batch: Batch) -> i32;
    fn llama_sampler_chain_init(params: SamplerChainParams) -> *mut c_void;
    fn llama_sampler_chain_add(chain: *mut c_void, sampler: *mut c_void);
    fn llama_sampler_init_greedy() -> *mut c_void;
    fn llama_sampler_sample(sampler: *mut c_void, context: *mut c_void, index: i32) -> i32;
    fn llama_sampler_accept(sampler: *mut c_void, token: i32);
    fn llama_sampler_free(sampler: *mut c_void);
    fn llama_vocab_is_eog(vocab: *const c_void, token: i32) -> bool;
    fn llama_token_to_piece(vocab: *const c_void, token: i32, buffer: *mut c_char, length: i32, lstrip: i32, special: bool) -> i32;
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
    active: Mutex<bool>,
}

impl LocalRuntimeService {
    pub(crate) fn run(&self, canonical_bytes: &[u8]) -> LocalRuntimeSnapshot {
        let Ok(mut active) = self.active.lock() else { return failed("runtime-unavailable"); };
        if *active { return failed("runtime-busy"); }
        *active = true;
        let result = run_once(canonical_bytes);
        *active = false;
        result
    }
}

fn run_once(canonical_bytes: &[u8]) -> LocalRuntimeSnapshot {
    let Ok(model_path) = std::env::var(MODEL_PATH_ENV) else { return failed("model-unavailable"); };
    if model_path.is_empty() || model_path.as_bytes().contains(&0) { return failed("model-unavailable"); }
    let Ok(path) = CString::new(model_path) else { return failed("model-unavailable"); };
    let prompt = match std::str::from_utf8(canonical_bytes) {
        Ok(value) => format!("You are a local, offline assistant. Answer the reviewed request only.\n\n{value}"),
        Err(_) => return failed("reviewed-input-invalid"),
    };
    let Ok(prompt) = CString::new(prompt) else { return failed("reviewed-input-invalid"); };
    let started = Instant::now();
    unsafe {
        llama_backend_init();
        let mut model_params = llama_model_default_params();
        model_params.n_gpu_layers = 0;
        model_params.split_mode = 0;
        let model = llama_model_load_from_file(path.as_ptr(), model_params);
        if model.is_null() { return failed("model-unavailable"); }
        let mut context_params = llama_context_default_params();
        context_params.n_ctx = INPUT_TOKEN_LIMIT as u32;
        context_params.n_batch = 512;
        context_params.n_ubatch = 512;
        context_params.n_threads = 1;
        context_params.n_threads_batch = 1;
        context_params.offload_kqv = false;
        context_params.op_offload = false;
        context_params.no_perf = true;
        let context = llama_init_from_model(model, context_params);
        if context.is_null() { llama_model_free(model); return failed("runtime-unavailable"); }
        let vocab = llama_model_get_vocab(model);
        let mut tokens = vec![0_i32; INPUT_TOKEN_LIMIT];
        let token_count = llama_tokenize(vocab, prompt.as_ptr(), prompt.as_bytes().len() as i32, tokens.as_mut_ptr(), tokens.len() as i32, true, true);
        if token_count <= 0 || token_count as usize >= INPUT_TOKEN_LIMIT { llama_free(context); llama_model_free(model); return failed("reviewed-input-too-large"); }
        tokens.truncate(token_count as usize);
        if llama_decode(context, llama_batch_get_one(tokens.as_mut_ptr(), token_count)) != 0 { llama_free(context); llama_model_free(model); return failed("runtime-failed"); }
        let sampler = llama_sampler_chain_init(llama_sampler_chain_default_params());
        if sampler.is_null() { llama_free(context); llama_model_free(model); return failed("runtime-unavailable"); }
        llama_sampler_chain_add(sampler, llama_sampler_init_greedy());
        let mut output = String::new();
        for _ in 0..OUTPUT_TOKEN_LIMIT {
            if started.elapsed() >= DEADLINE { llama_sampler_free(sampler); llama_free(context); llama_model_free(model); return failed("deadline-exceeded"); }
            let token = llama_sampler_sample(sampler, context, -1);
            if llama_vocab_is_eog(vocab, token) { break; }
            let mut piece = [0_i8; 256];
            let piece_len = llama_token_to_piece(vocab, token, piece.as_mut_ptr(), piece.len() as i32, 0, false);
            if piece_len <= 0 || output.len().saturating_add(piece_len as usize) > OUTPUT_BYTE_LIMIT { break; }
            output.push_str(&String::from_utf8_lossy(std::slice::from_raw_parts(piece.as_ptr().cast::<u8>(), piece_len as usize)));
            llama_sampler_accept(sampler, token);
            let mut next = [token];
            if llama_decode(context, llama_batch_get_one(next.as_mut_ptr(), 1)) != 0 { llama_sampler_free(sampler); llama_free(context); llama_model_free(model); return failed("runtime-failed"); }
        }
        llama_sampler_free(sampler);
        llama_free(context);
        llama_model_free(model);
        LocalRuntimeSnapshot { schema_version: 1, local_only: true, state: "completed".into(), output: Some(output), diagnostic: None, input_token_limit: INPUT_TOKEN_LIMIT as u16, output_token_limit: OUTPUT_TOKEN_LIMIT as u16, deadline_seconds: 60 }
    }
}

fn failed(diagnostic: &str) -> LocalRuntimeSnapshot {
    LocalRuntimeSnapshot { schema_version: 1, local_only: true, state: "failed".into(), output: None, diagnostic: Some(diagnostic.into()), input_token_limit: INPUT_TOKEN_LIMIT as u16, output_token_limit: OUTPUT_TOKEN_LIMIT as u16, deadline_seconds: 60 }
}
