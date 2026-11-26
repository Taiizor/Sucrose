//! The Chromium switches model (D-17, WAPI-10): the two legacy shapes convert into each other, Reset gives the legacy
//! defaults, Sucrose's own switches are computed and merged without repeating the feature lists.

use std::{collections::BTreeMap, fs, path::Path};

use sucrose_engine_web::args::{
    ArgError, CEF_SETTING_KEYS, SucroseInputs, Switch, WebEngine, browser_arguments, debugging_port,
    effective_switches, from_cef_map, from_web_list, legacy_defaults_cef, legacy_defaults_web, merge,
    plan_cef_command_line, plan_webview2_command_line, repeated_names, sucrose_switches, to_cef_map, to_web_list,
    validate_name,
};

fn inputs(hardware_acceleration: bool, windows_build: u32, engine: WebEngine) -> SucroseInputs {
    SucroseInputs {
        hardware_acceleration,
        debugging_port: None,
        windows_build,
        engine,
    }
}

fn names(switches: &[Switch]) -> Vec<&str> {
    switches.iter().map(|switch| switch.name.as_str()).collect()
}

#[test]
fn args_cef_defaults_survive_the_round_trip_and_reset_twice_is_the_same() {
    let defaults = legacy_defaults_cef();

    let back = to_cef_map(&from_cef_map(&defaults));
    let again = to_cef_map(&from_cef_map(&back));

    assert_eq!(back, defaults);
    assert_eq!(again, back, "converting twice gives the same value");
    assert_eq!(legacy_defaults_cef(), defaults, "Reset twice gives the same value");
}

#[test]
fn args_web_defaults_survive_the_round_trip_and_reset_twice_is_the_same() {
    let defaults = legacy_defaults_web();

    let back = to_web_list(&from_web_list(&defaults));
    let again = to_web_list(&from_web_list(&back));

    assert_eq!(back, defaults);
    assert_eq!(again, back);
    assert_eq!(legacy_defaults_web(), defaults);
    assert!(defaults.contains(&"--autoplay-policy=no-user-gesture-required".to_owned()));
    assert!(defaults.contains(&"--no-sandbox".to_owned()));
}

/// The quoted strings of the first block of `Internal.cs` that starts with `marker`, commented lines left out.
fn legacy_block(source: &str, marker: &str) -> Vec<String> {
    let start = source.find(marker).expect("the legacy block exists");
    let block = &source[start..];
    let end = block.find("];").or_else(|| block.find("};")).expect("the block ends");
    block[..end]
        .lines()
        .skip(1)
        .map(str::trim)
        .filter(|line| !line.starts_with("//") && line.contains('"'))
        .map(str::to_owned)
        .collect()
}

#[test]
fn args_legacy_defaults_equal_the_lists_of_the_legacy_source() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src/Shared/Engine/Sucrose.Shared.Engine/Manage/Internal.cs");
    let Ok(source) = fs::read_to_string(&path) else {
        // The legacy tree is removed at parity sign-off (M7); until then it is the reference these lists copy.
        return;
    };

    let web: Vec<String> = legacy_block(&source, "List<string> WebArguments")
        .iter()
        .map(|line| line.trim_end_matches(',').trim().trim_matches('"').to_owned())
        .collect();
    let cef: BTreeMap<String, String> = legacy_block(&source, "Dictionary<string, string> CefArguments")
        .iter()
        .map(|line| {
            let (key, value) = line.split_once("] =").expect("a dictionary entry");
            (
                key.trim().trim_start_matches('[').trim_matches('"').to_owned(),
                value.trim().trim_end_matches(',').trim_matches('"').to_owned(),
            )
        })
        .collect();

    assert!(web.len() > 30 && cef.len() > 30, "{} {}", web.len(), cef.len());
    assert_eq!(legacy_defaults_web(), web);
    assert_eq!(legacy_defaults_cef(), cef);
}

#[test]
fn args_entries_with_and_without_a_value_parse() {
    let list = [
        "--no-sandbox",
        "--autoplay-policy=no-user-gesture-required",
        "-x=1",
        "bare",
        "  ",
        "--",
    ]
    .map(str::to_owned);

    let parsed = from_web_list(&list);

    assert_eq!(
        parsed,
        [
            Switch::flag("no-sandbox"),
            Switch::with_value("autoplay-policy", "no-user-gesture-required"),
            Switch::with_value("x", "1"),
            Switch::flag("bare"),
        ]
    );
    let map = BTreeMap::from([("a".to_owned(), String::new()), ("b".to_owned(), "2".to_owned())]);
    assert_eq!(from_cef_map(&map), [Switch::flag("a"), Switch::with_value("b", "2")]);
    assert_eq!(to_cef_map(&from_cef_map(&map)), map);
}

#[test]
fn args_names_with_spaces_or_other_characters_are_refused_and_repeats_are_reported() {
    assert_eq!(validate_name("enable-features"), Ok(()));
    assert_eq!(validate_name("Use2"), Ok(()));
    assert_eq!(validate_name(""), Err(ArgError::EmptyName));
    for bad in ["has space", "a=b", "a;b", "--lead", "naïve", "a\nb"] {
        assert_eq!(
            validate_name(bad),
            Err(ArgError::InvalidName(bad.to_owned())),
            "{bad:?}"
        );
    }

    let switches = [
        Switch::flag("a"),
        Switch::flag("b"),
        Switch::flag("a"),
        Switch::flag("a"),
    ];
    assert_eq!(repeated_names(&switches), ["a"]);
    assert!(repeated_names(&[Switch::flag("a"), Switch::flag("b")]).is_empty());
}

#[test]
fn args_hardware_acceleration_adds_the_gpu_switches_by_engine_and_build() {
    let on = sucrose_switches(&inputs(true, 22631, WebEngine::Cef));
    assert!(
        names(&on).contains(&"enable-gpu") && names(&on).contains(&"enable-gpu-vsync"),
        "{on:?}"
    );

    let off_old = sucrose_switches(&inputs(false, 22631, WebEngine::Cef));
    assert!(names(&off_old).contains(&"disable-gpu") && names(&off_old).contains(&"disable-gpu-vsync"));

    let off_new = sucrose_switches(&inputs(false, 26300, WebEngine::Cef));
    assert!(
        off_new.is_empty(),
        "build 26000 and later add no disable switch for CEF: {off_new:?}"
    );

    let off_web = sucrose_switches(&inputs(false, 26300, WebEngine::WebView2));
    assert_eq!(
        to_web_list(&off_web),
        ["--disable-gpu", "--disable-gpu-vsync"],
        "WebView2 always gets them"
    );
}

#[test]
fn args_a_debugging_port_adds_the_origin_rule_and_webview_also_the_port() {
    let mut cef = inputs(true, 22631, WebEngine::Cef);
    cef.debugging_port = Some(9222);
    assert!(sucrose_switches(&cef).contains(&Switch::with_value("remote-allow-origins", "*")));
    assert!(
        !names(&sucrose_switches(&cef)).contains(&"remote-debugging-port"),
        "CEF takes it from settings"
    );

    let mut web = inputs(true, 22631, WebEngine::WebView2);
    web.debugging_port = Some(9300);
    let list = to_web_list(&sucrose_switches(&web));
    assert!(list.contains(&"--remote-debugging-port=9300".to_owned()), "{list:?}");
    assert!(list.contains(&"--remote-allow-origins=*".to_owned()));
    assert!(list.contains(&"--enable-features=msEdgeDevToolsWdpRemoteDebugging".to_owned()));

    assert!(!names(&sucrose_switches(&inputs(true, 22631, WebEngine::Cef))).contains(&"remote-allow-origins"));
}

#[test]
fn args_each_renderer_gets_its_own_debugging_port_and_a_bad_sum_is_refused() {
    assert_eq!(debugging_port(0, 3), Ok(None));
    assert_eq!(debugging_port(9222, 0), Ok(Some(9222)));
    assert_eq!(debugging_port(9222, 2), Ok(Some(9224)));
    assert_eq!(
        debugging_port(65535, 1),
        Err(ArgError::PortOutOfRange { base: 65535, slot: 1 })
    );
    assert!(debugging_port(80, 0).is_err(), "CEF ignores ports below 1024");
}

#[test]
fn args_feature_lists_are_merged_into_one_switch() {
    let user = [Switch::with_value("enable-features", "A,B"), Switch::flag("no-sandbox")];
    let sucrose = [Switch::with_value("enable-features", "C,A")];

    let merged = merge(&user, &sucrose);

    assert_eq!(
        merged,
        [
            Switch::with_value("enable-features", "A,B,C"),
            Switch::flag("no-sandbox")
        ]
    );
    let two_user = merge(
        &[
            Switch::with_value("disable-features", "X"),
            Switch::with_value("disable-features", "Y,X"),
        ],
        &[],
    );
    assert_eq!(two_user, [Switch::with_value("disable-features", "X,Y")]);
}

#[test]
fn args_a_user_switch_that_sucrose_sets_keeps_sucroses_value() {
    let user = [
        Switch::with_value("enable-gpu", "0"),
        Switch::flag("keep"),
        Switch::with_value("enable-gpu", "2"),
    ];

    let merged = merge(&user, &[Switch::with_value("enable-gpu", "1")]);

    assert_eq!(merged, [Switch::with_value("enable-gpu", "1"), Switch::flag("keep")]);
    assert_eq!(
        merge(&user, &[]),
        user,
        "without a Sucrose switch the user's stay as typed"
    );
}

#[test]
fn args_the_cef_plan_skips_the_setting_keys_and_keeps_the_legacy_switches() {
    let user = from_cef_map(&legacy_defaults_cef());

    let plan = plan_cef_command_line(&user, &inputs(true, 22631, WebEngine::Cef));

    assert_eq!(plan.skipped_settings, CEF_SETTING_KEYS.map(str::to_owned));
    assert!(plan.invalid.is_empty());
    assert!(
        plan.applied
            .contains(&Switch::with_value("autoplay-policy", "no-user-gesture-required"))
    );
    assert!(plan.applied.contains(&Switch::with_value("enable-gpu", "1")));
    assert!(!names(&plan.applied).iter().any(|name| CEF_SETTING_KEYS.contains(name)));
    assert_eq!(
        names(&plan.applied)
            .iter()
            .filter(|name| **name == "enable-features")
            .count(),
        1,
        "never repeated"
    );
}

#[test]
fn args_the_cef_plan_leaves_out_invalid_names_and_passes_unknown_valid_ones() {
    let user = [
        Switch::flag("not a name"),
        Switch::with_value("some-future-switch", "x y"),
    ];

    let plan = plan_cef_command_line(&user, &inputs(true, 22631, WebEngine::Cef));

    assert_eq!(plan.invalid.len(), 1);
    assert_eq!(plan.invalid[0].0, "not a name");
    assert!(plan.applied.contains(&Switch::with_value("some-future-switch", "x y")));
}

#[test]
fn args_windows_11_24h2_drops_the_users_gpu_compositing_switch_for_cef_only() {
    let user = [Switch::with_value("disable-gpu-compositing", "1")];

    let new = effective_switches(&user, &inputs(true, 26100, WebEngine::Cef));
    let old = effective_switches(&user, &inputs(true, 22631, WebEngine::Cef));
    let web = effective_switches(&user, &inputs(true, 26100, WebEngine::WebView2));

    assert!(!names(&new).contains(&"disable-gpu-compositing"));
    assert!(names(&old).contains(&"disable-gpu-compositing"));
    assert!(names(&web).contains(&"disable-gpu-compositing"));
}

fn text_of(user: &[&str], hardware_acceleration: bool, port: Option<u16>) -> String {
    let user = from_web_list(&user.iter().map(|entry| (*entry).to_owned()).collect::<Vec<_>>());
    let mut inputs = inputs(hardware_acceleration, 26100, WebEngine::WebView2);
    inputs.debugging_port = port;
    browser_arguments(&plan_webview2_command_line(&user, &inputs).applied)
}

#[test]
fn args_the_webview2_text_is_one_line_of_the_user_switches_and_sucroses_own() {
    let text = text_of(
        &["--autoplay-policy=no-user-gesture-required", "--disable-web-security"],
        true,
        None,
    );

    assert_eq!(
        text,
        "--autoplay-policy=no-user-gesture-required --disable-web-security --enable-gpu --enable-gpu-vsync"
    );
}

#[test]
fn args_the_webview2_text_merges_the_feature_lists_into_one_switch_with_a_port() {
    let text = text_of(&["--enable-features=Foo,Bar"], false, Some(9223));

    assert_eq!(
        text,
        "--enable-features=Foo,Bar,msEdgeDevToolsWdpRemoteDebugging --disable-gpu --disable-gpu-vsync \
         --remote-debugging-port=9223 --remote-allow-origins=*"
    );
    assert_eq!(text.matches("enable-features").count(), 1);
}

#[test]
fn args_a_webview2_value_with_a_blank_is_quoted_and_a_quote_inside_is_escaped() {
    let switches = [
        Switch::with_value("user-agent", "My Agent 1.0"),
        Switch::with_value("note", "say \"hi\" now"),
        Switch::with_value("path", "C:\\a b\\"),
        Switch::with_value("plain", "x"),
        Switch::flag("flag"),
    ];

    assert_eq!(
        browser_arguments(&switches),
        r#"--user-agent="My Agent 1.0" --note="say \"hi\" now" --path="C:\a b\\" --plain=x --flag"#
    );
    assert_eq!(browser_arguments(&[]), "");
}

#[test]
fn args_the_webview2_plan_leaves_out_invalid_names() {
    let user = [Switch::flag("good"), Switch::flag("bad name --evil"), Switch::flag("")];
    let plan = plan_webview2_command_line(&user, &inputs(true, 0, WebEngine::WebView2));

    assert_eq!(names(&plan.applied), ["good", "enable-gpu", "enable-gpu-vsync"]);
    assert_eq!(plan.invalid.len(), 2);
    assert!(!browser_arguments(&plan.applied).contains("evil"));
}

#[test]
fn args_the_webview2_debugging_port_is_the_base_plus_the_slot_and_out_of_range_leaves_it_off() {
    assert_eq!(debugging_port(9222, 1), Ok(Some(9223)));
    assert!(debugging_port(65535, 1).is_err());
    assert!(debugging_port(1000, 0).is_err());
    assert_eq!(debugging_port(0, 3), Ok(None));
}
