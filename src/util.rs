use anyhow::{Context, anyhow, Error};
use napi::{CallContext, JsBuffer, Status};
use serde::de::DeserializeOwned;
use std::{
    any::type_name,
    panic::{catch_unwind, AssertUnwindSafe},
};
use swc::{try_with_handler};
use swc_common::{
    errors::{Handler, ColorConfig},
    sync::{Lrc},
    SourceMap,
};

use crate::TransformOutput;

pub trait CtxtExt {
    fn get_buffer_as_string(&self, index: usize) -> napi::Result<String>;
    /// Currently this uses JsBuffer
    fn get_deserialized<T>(&self, index: usize) -> napi::Result<T>
    where
        T: DeserializeOwned;
}

impl CtxtExt for CallContext<'_> {
    fn get_buffer_as_string(&self, index: usize) -> napi::Result<String> {
        let buffer = self.get::<JsBuffer>(index)?.into_value()?;

        Ok(String::from_utf8_lossy(buffer.as_ref()).to_string())
    }

    fn get_deserialized<T>(&self, index: usize) -> napi::Result<T>
    where
        T: DeserializeOwned,
    {
        let buffer = self.get::<JsBuffer>(index)?.into_value()?;
        let v = serde_json::from_slice(&buffer)
            .with_context(|| {
                format!(
                    "Failed to deserialize argument at `{}` as {}\nJSON: {}",
                    index,
                    type_name::<T>(),
                    String::from_utf8_lossy(&buffer)
                )
            })
            .convert_err()?;

        Ok(v)
    }
}

pub trait MapErr<T>: Into<Result<T, anyhow::Error>> {
    fn convert_err(self) -> napi::Result<T> {
        self.into()
            .map_err(|err| napi::Error::new(Status::GenericFailure, format!("{:?}", err)))
    }
}

impl<T> MapErr<T> for Result<T, anyhow::Error> {}

pub(crate) fn get_deserialized<T, B>(buffer: B) -> napi::Result<T>
where
    T: DeserializeOwned,
    B: AsRef<[u8]>,
{
    let mut deserializer = serde_json::Deserializer::from_slice(buffer.as_ref());
    deserializer.disable_recursion_limit();

    let v = T::deserialize(&mut deserializer)
        .with_context(|| {
            format!(
                "Failed to deserialize buffer as {}\nJSON: {}",
                type_name::<T>(),
                String::from_utf8_lossy(buffer.as_ref())
            )
        })
        .convert_err()?;

    Ok(v)
}

pub(crate) fn deserialize_json<T>(json: &str) -> Result<T, serde_json::Error>
where
    T: DeserializeOwned,
{
    let mut deserializer = serde_json::Deserializer::from_str(&json);
    deserializer.disable_recursion_limit();

    T::deserialize(&mut deserializer)
}

pub fn try_with<F, Ret>(cm: Lrc<SourceMap>, op: F) -> Result<Ret, Error>
where
    F: FnOnce(&Handler) -> Result<Ret, Error>,
{
    try_with_handler(
        cm,
        swc::HandlerOpts {
            color: ColorConfig::Never,
            skip_filename: true,
            ..Default::default()
        },
        |handler| {
            //
            let result = catch_unwind(AssertUnwindSafe(|| op(handler)));

            let p = match result {
                Ok(v) => return v,
                Err(v) => v,
            };

            if let Some(s) = p.downcast_ref::<String>() {
                Err(anyhow!("failed to handle: {}", s))
            } else if let Some(s) = p.downcast_ref::<&str>() {
                Err(anyhow!("failed to handle: {}", s))
            } else {
                Err(anyhow!("failed to handle with unknown panic message"))
            }
        },
    )
}

pub fn format_output(result: Result<swc::TransformOutput, Error>) -> Result<TransformOutput, napi::Error> {
    match result {
        Ok(v) => return Ok(TransformOutput {
            code: v.code,
            map: v.map
        }),
        Err(v) => return Err(v).convert_err()
    }
}
