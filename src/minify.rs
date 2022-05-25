use crate::{
    get_compiler,
    util::{deserialize_json, get_deserialized, try_with, format_output},
    TransformOutput
};
use napi::{
    bindgen_prelude::{AbortSignal, AsyncTask, Buffer},
    Task,
};
use napi_derive::napi;
use serde::Deserialize;
use std::sync::Arc;
use swc::{config::JsMinifyOptions};
use swc_common::{collections::AHashMap, sync::Lrc, FileName, SourceFile, SourceMap};

pub struct MinifyTask {
    c: Arc<swc::Compiler>,
    code: String,
    options: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum MinifyTarget {
    /// Code to minify.
    Single(String),
    /// `{ filename: code }`
    Map(AHashMap<String, String>),
}

impl MinifyTarget {
    fn to_file(&self, cm: Lrc<SourceMap>) -> Lrc<SourceFile> {
        match self {
            MinifyTarget::Single(code) => cm.new_source_file(FileName::Anon, code.clone()),
            MinifyTarget::Map(codes) => {
                assert_eq!(
                    codes.len(),
                    1,
                    "swc.minify does not support concatting multiple files yet"
                );

                let (filename, code) = codes.iter().next().unwrap();

                cm.new_source_file(FileName::Real(filename.clone().into()), code.clone())
            }
        }
    }
}

#[napi]
impl Task for MinifyTask {
    type Output = TransformOutput;

    type JsValue = TransformOutput;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let input: MinifyTarget = deserialize_json(&self.code)?;
        let options: JsMinifyOptions = deserialize_json(&self.options)?;

        let result = try_with(self.c.cm.clone(), |handler| {
            let fm = input.to_file(self.c.cm.clone());

            self.c.minify(fm, &handler, &options)
        });

        format_output(result)
    }

    fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

#[napi]
pub fn minify(code: Buffer, opts: Buffer, signal: Option<AbortSignal>) -> AsyncTask<MinifyTask> {
    let code = String::from_utf8_lossy(code.as_ref()).to_string();
    let options = String::from_utf8_lossy(opts.as_ref()).to_string();

    let c = get_compiler();

    let task = MinifyTask { c, code, options };

    AsyncTask::with_optional_signal(task, signal)
}

#[napi]
pub fn minify_sync(code: Buffer, opts: Buffer) -> napi::Result<TransformOutput> {
    let code: MinifyTarget = get_deserialized(&code)?;
    let opts = get_deserialized(&opts)?;

    let c = get_compiler();

    let fm = code.to_file(c.cm.clone());

    let result = try_with(c.cm.clone(), |handler| c.minify(fm, &handler, &opts));

    format_output(result)
}
