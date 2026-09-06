use butterfly_guard::{parse_config, Debouncer};
#[test]
fn settings_round_trip_preserves_global_and_key_exemptions() {
    let config = parse_config(r#"{"delay_ms":60,"key_delays_ms":{"30":80,"42":0}}"#).unwrap();
    assert_eq!(parse_config(&serde_json::to_string(&config).unwrap()).unwrap(),config);
    let mut d = Debouncer::new(&config);
    for key in [30,42,48] { assert!(d.accept(key,1,0)); assert!(d.accept(key,0,1)); }
    assert!(!d.accept(30,1,70000)); // A override is 80 ms
    assert!(d.accept(48,1,70000)); // B inherits 60 ms
    assert!(d.accept(42,1,2)); // Shift is exempt
}
#[test]
fn settings_reject_unknown_fields_and_invalid_key_delays() {
    for value in [r#"{"delay_ms":40,"key_delays_ms":{"30":201}}"#,r#"{"delay_ms":40,"key_delays_ms":{},"command":"anything"}"#,r#"{"delay_ms":40,"key_delays_ms":{"0":40}}"#] {
        assert!(parse_config(value).is_err());
    }
}
