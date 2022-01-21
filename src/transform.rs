use crate::{
    custom_before_pass, get_compiler,
    util::{deserialize_json, get_deserialized, MapErr},
    TransformOptions,
};
use napi::{
    bindgen_prelude::{AbortSignal, AsyncTask, Buffer},
    Env, Task,
};
use napi_derive::napi;
use std::{path::Path, sync::Arc};
use swc::{try_with_handler, Compiler, TransformOutput};
use swc_common::FileName;
use swc_ecmascript::transforms::pass::noop;

/// Input to transform
#[derive(Debug)]
pub enum Input {
    /// Raw source code.
    Source { src: String },
}

pub struct TransformTask {
    pub c: Arc<Compiler>,
    pub input: Input,
    pub options: String,
}

#[napi]
impl Task for TransformTask {
    type Output = TransformOutput;
    type JsValue = TransformOutput;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let mut options: TransformOptions = deserialize_json(&self.options)?;
        if !options.swc.filename.is_empty() {
            options.swc.config.adjust(Path::new(&options.swc.filename));
        }

        try_with_handler(
            self.c.cm.clone(),
            !options.swc.config.error.filename,
            |handler| {
                self.c.run(|| match &self.input {
                    Input::Source { src } => {
                        let fm = self.c.cm.new_source_file(
                            if options.swc.filename.is_empty() {
                                FileName::Anon
                            } else {
                                FileName::Real(options.swc.filename.clone().into())
                            },
                            src.to_string(),
                        );

                        let before_pass = custom_before_pass(&fm.name, &options);
                        self.c.process_js_with_custom_pass(
                            fm.clone(),
                            None,
                            &handler,
                            &options.swc,
                            |_| before_pass,
                            |_| noop(),
                        )
                    }
                })
            },
        )
        .convert_err()
    }

    fn resolve(&mut self, _env: Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
}

#[napi]
pub fn transform(
    src: String,
    options: Buffer,
    signal: Option<AbortSignal>,
) -> napi::Result<AsyncTask<TransformTask>> {
    let c = get_compiler();

    let options = String::from_utf8_lossy(options.as_ref()).to_string();

    let input = Input::Source { src };

    let task = TransformTask {
        c: c.clone(),
        input,
        options,
    };
    Ok(AsyncTask::with_optional_signal(task, signal))
}

#[napi]
pub fn transform_sync(s: String, opts: Buffer) -> napi::Result<TransformOutput> {
    let c = get_compiler();

    let mut options: TransformOptions = get_deserialized(&opts)?;

    if !options.swc.filename.is_empty() {
        options.swc.config.adjust(Path::new(&options.swc.filename));
    }

    try_with_handler(
        c.cm.clone(),
        !options.swc.config.error.filename,
        |handler| {
            c.run(|| {
                let fm = c.cm.new_source_file(
                    if options.swc.filename.is_empty() {
                        FileName::Anon
                    } else {
                        FileName::Real(options.swc.filename.clone().into())
                    },
                    s,
                );
                let before_pass = custom_before_pass(&fm.name, &options);
                c.process_js_with_custom_pass(
                    fm.clone(),
                    None,
                    &handler,
                    &options.swc,
                    |_| before_pass,
                    |_| noop(),
                )
            })
        },
    )
    .convert_err()
}

#[test]
fn test_deser() {
    const JSON_STR: &str = r#"{"jsc":{"parser":{"syntax":"ecmascript","dynamicImport":true,"jsx":true},"transform":{"react":{"runtime":"automatic","pragma":"React.createElement","pragmaFrag":"React.Fragment","throwIfNamespace":true,"development":false,"useBuiltins":true}},"target":"es5"},"filename":"/Users/filename","sourceMaps":false,"sourceFileName":"/Users/sourceFilename" }"#;

    let tr: TransformOptions = serde_json::from_str(&JSON_STR).unwrap();

    println!("{:#?}", tr);
}

#[test]
fn test_deserialize_transform_regenerator() {
    const JSON_STR: &str = r#"{"jsc":{"parser":{"syntax":"ecmascript","dynamicImport":true,"jsx":true},"transform":{ "regenerator": { "importPath": "foo" }, "react":{"runtime":"automatic","pragma":"React.createElement","pragmaFrag":"React.Fragment","throwIfNamespace":true,"development":false,"useBuiltins":true}},"target":"es5"},"filename":"/Users/sourceFilename","sourceMaps":false,"sourceFileName":"/Users/filename" }"#;

    let tr: TransformOptions = serde_json::from_str(&JSON_STR).unwrap();

    println!("{:#?}", tr);
}
