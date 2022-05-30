use builder_swc::{
    keep_platform::{keep_platform, KeepPlatformConfig},
    remove_export_exprs::remove_export_exprs,
};
use std::path::PathBuf;
use swc_ecma_transforms_testing::{test, test_fixture};
use swc_ecmascript::{
    parser::{EsConfig, Syntax},
    transforms::react::jsx,
};
use swc_common::{
    Mark,
    comments::SingleThreadedComments,
    chain,
};
use testing::fixture;

fn unminify_syntax() -> Syntax {
    Syntax::Es(EsConfig {
        jsx: true,
        ..Default::default()
    })
}

#[fixture("tests/fixture/keep_platform/web/input.js")]
fn transform_web_flag_fixture(input: PathBuf) {
    let output = input.parent().unwrap().join("output.js");
    let config = KeepPlatformConfig::KeepPlatform(String::from("web"));
    test_fixture(
        unminify_syntax(),
        &|_tr| keep_platform(config.clone()),
        &input,
        &output,
    );
}

#[fixture("tests/fixture/keep_platform/kraken/input.js")]
fn transform_kraken_flag_fixture(input: PathBuf) {
    let output = input.parent().unwrap().join("output.js");
    let config = KeepPlatformConfig::KeepPlatform(String::from("kraken"));
    test_fixture(
        unminify_syntax(),
        &|_tr| keep_platform(config.clone()),
        &input,
        &output,
    );
}

#[fixture("tests/fixture/keep_platform/namespace/web/input.js")]
fn transform_namespace_web_fixture(input: PathBuf) {
    let output = input.parent().unwrap().join("output.js");
    let config = KeepPlatformConfig::KeepPlatform(String::from("web"));
    test_fixture(
        unminify_syntax(),
        &|_tr| keep_platform(config.clone()),
        &input,
        &output,
    );
}

#[fixture("tests/fixture/keep_platform/namespace/kraken/input.js")]
fn transform_namespace_kraken_fixture(input: PathBuf) {
    let output = input.parent().unwrap().join("output.js");
    let config = KeepPlatformConfig::KeepPlatform(String::from("kraken"));
    test_fixture(
        unminify_syntax(),
        &|_tr| keep_platform(config.clone()),
        &input,
        &output,
    );
}

#[fixture("tests/fixture/remove_export_exprs/**/input.js")]
fn remove_export_exprs_fixture(input: PathBuf) {
    let output = input.parent().unwrap().join("output.js");
    test_fixture(
        unminify_syntax(),
        &|tr| {
            let top_level_mark = Mark::fresh(Mark::root());
            let jsx = jsx::<SingleThreadedComments>(
                tr.cm.clone(),
                None,
                swc_ecmascript::transforms::react::Options {
                    next: false.into(),
                    runtime: None,
                    import_source: Some("".into()),
                    pragma: Some("__jsx".into()),
                    pragma_frag: Some("__jsxFrag".into()),
                    throw_if_namespace: false.into(),
                    development: false.into(),
                    use_builtins: false.into(),
                    use_spread: true.into(),
                    refresh: Default::default(),
                },
                top_level_mark,
            );
            chain!(remove_export_exprs(vec!["getData".to_string(), "default".to_string()]), jsx)
        },
        &input,
        &output,
    );
}
