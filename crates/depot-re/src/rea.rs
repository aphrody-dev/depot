// SPDX-License-Identifier: Apache-2.0 OR MIT
//! REA (Runtime Execution Analysis) and MinGW-w64 CRT boilerplate noise filter.

use serde::{Deserialize, Serialize};

/// Canonical MinGW-w64 CRT startup, TLS, SEH, and pseudo-reloc boilerplate symbols.
pub const MINGW_CRT_NOISE_SYMBOLS: &[&str] = &[
    "mainCRTStartup",
    "wmainCRTStartup",
    "WinMainCRTStartup",
    "wWinMainCRTStartup",
    "__tmainCRTStartup",
    "DllMainCRTStartup",
    "_DllMainCRTStartup",
    "_pei386_runtime_relocator",
    "__mingw_CRTStartup",
    "__mingw_init_ehandler",
    "__mingw_SEH_error_handler",
    "__mingw_TLScallback",
    "__dyn_tls_init",
    "__tlregdtor",
    "__main",
    "__do_global_ctors",
    "__do_global_dtors",
    "___chkstk_ms",
    "__chkstk_ms",
    "_gnu_exception_handler",
    "__gcc_register_frame",
    "__gcc_deregister_frame",
    "atexit",
    "_onexit",
    "pre_c_init",
    "pre_cpp_init",
    "check_managed_app",
    "duplicate_ppstrings",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaNoiseFilterReport {
    pub total_symbols: usize,
    pub noise_symbols_count: usize,
    pub payload_symbols_count: usize,
    pub noise_reduction_ratio: f64,
    pub payload_symbols: Vec<String>,
}

/// Filters toolchain boilerplate symbols to isolate the genuine user payload functions.
pub fn filter_crt_boilerplate(symbols: &[String]) -> ReaNoiseFilterReport {
    let mut payload = Vec::new();
    let mut noise_count = 0usize;

    for s in symbols {
        let is_noise = MINGW_CRT_NOISE_SYMBOLS.iter().any(|&n| s.contains(n))
            || s.starts_with("__imp_")
            || s.starts_with("__mingw_");
        if is_noise {
            noise_count += 1;
        } else {
            payload.push(s.clone());
        }
    }

    let total = symbols.len();
    let ratio = if total > 0 {
        noise_count as f64 / total as f64
    } else {
        0.0
    };

    ReaNoiseFilterReport {
        total_symbols: total,
        noise_symbols_count: noise_count,
        payload_symbols_count: payload.len(),
        noise_reduction_ratio: ratio,
        payload_symbols: payload,
    }
}
