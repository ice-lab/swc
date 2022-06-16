use crate::{
    custom_before_pass, get_compiler,
    util::{get_deserialized, try_with, format_output},
    TransformOptions,
    TransformOutput,
};
use napi::{
    bindgen_prelude::{AbortSignal, AsyncTask, Buffer},
    Env, Task, JsBufferValue, Ref, JsBuffer
};
use napi_derive::napi;
use std::{path::Path, sync::Arc};
use swc::{Compiler};
use swc_common::{FileName};
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
    pub options: Ref<JsBufferValue>,
}

#[napi]
impl Task for TransformTask {
    type JsValue = TransformOutput;
    type Output = TransformOutput;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let mut options: TransformOptions = serde_json::from_slice(self.options.as_ref())?;
        if !options.swc.filename.is_empty() {
            options.swc.config.adjust(Path::new(&options.swc.filename));
        }

        let result = try_with(
            self.c.cm.clone(),
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

                        let file = fm.clone();

                        self.c.process_js_with_custom_pass(
                            file,
                            None,
                            handler,
                            &options.swc,
                            |_, _| {
                                custom_before_pass(
                                    &fm.name,
                                    &options
                                )
                            } ,
                            |_, _| noop(),
                        )
                    }
                })
            },
        );

        format_output(result)
    }

    fn resolve(&mut self, _env: Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
    fn finally(&mut self, _env: Env) -> napi::Result<()> {
        Ok(())
    }
}

#[napi]
pub fn transform(
    src: String,
    options: JsBuffer,
    signal: Option<AbortSignal>,
) -> napi::Result<AsyncTask<TransformTask>> {
    let c = get_compiler();

    let input = Input::Source { src };

    let task = TransformTask {
        c: c.clone(),
        input,
        options: options.into_ref()?,
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

    let res = try_with(
        c.cm.clone(),
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
                    |_, _| before_pass,
                    |_, _| noop(),
                )
            })
        },
    );

    format_output(res)
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
