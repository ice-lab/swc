#![recursion_limit = "2048"]
//#![deny(clippy::all)]

extern crate napi_derive;
extern crate lazy_static;
/// Explicit extern crate to use allocator.
extern crate swc_node_base;

use backtrace::Backtrace;
use lazy_static::lazy_static;
use napi_derive::napi;
use serde::Deserialize;
use std::{env, panic::set_hook, sync::Arc};
use swc::Compiler;
use swc_common::{self, chain, pass::Optional, sync::Lazy, FileName, FilePathMapping, SourceMap};
use swc_ecmascript::transforms::pass::noop;
use swc_ecmascript::visit::Fold;
use tracing_subscriber::filter::EnvFilter;

use crate::keep_platform::{keep_platform, KeepPlatformConfig};

pub mod keep_platform;
pub mod minify;
pub mod transform;
mod util;

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransformOptions {
    #[serde(flatten)]
    pub swc: swc::config::Options,
    #[serde(default)]
    pub keep_platform: KeepPlatformConfig,
}

pub fn custom_before_pass(_name: &FileName, options: &TransformOptions) -> impl Fold {
    let mut keep_platform_config = KeepPlatformConfig::Bool(false);
    let enable_keep_platform: bool = match options.keep_platform.clone() {
        KeepPlatformConfig::KeepPlatform(platform) => {
            keep_platform_config = KeepPlatformConfig::KeepPlatform(platform);
            true
        }
        KeepPlatformConfig::Bool(val) => val,
    };

    // custom before pass
    chain!(
        Optional::new(keep_platform(keep_platform_config), enable_keep_platform),
        noop()
    )
}

static COMPILER: Lazy<Arc<Compiler>> = Lazy::new(|| {
    let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));

    Arc::new(Compiler::new(cm.clone()))
});

#[napi::module_init]
fn init() {
    if cfg!(debug_assertions) || env::var("SWC_DEBUG").unwrap_or_default() == "1" {
        set_hook(Box::new(|panic_info| {
            let backtrace = Backtrace::new();
            println!("Panic: {:?}\nBacktrace: {:?}", panic_info, backtrace);
        }));
    }

    let _ = tracing_subscriber::FmtSubscriber::builder()
        .without_time()
        .with_target(false)
        .with_ansi(true)
        .with_env_filter(EnvFilter::from_env("SWC_LOG"))
        .try_init();
}

pub fn get_compiler() -> Arc<Compiler> {
    COMPILER.clone()
}

#[napi(js_name = "Compiler")]
pub struct JsCompiler {
    _compiler: Arc<Compiler>,
}

#[napi]
impl JsCompiler {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            _compiler: COMPILER.clone(),
        }
    }
}

pub type ArcCompiler = Arc<Compiler>;

#[napi(object)]
pub struct TransformOutput {
    pub code: String,
    pub map: Option<String>,
}
