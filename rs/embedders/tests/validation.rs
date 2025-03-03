use assert_matches::assert_matches;
use ic_config::{
    embedders::{Config as EmbeddersConfig, FeatureFlags},
    feature_status::FeatureStatus,
};
use ic_embedders::wasm_utils::validation::{
    validate_wasm_binary, WasmImportsDetails, WasmValidationDetails, RESERVED_SYMBOLS,
};
use ic_wasm_types::{BinaryEncodedWasm, WasmValidationError};

fn wat2wasm(wat: &str) -> Result<BinaryEncodedWasm, wabt::Error> {
    let mut features = wabt::Features::new();
    features.enable_multi_value();
    wabt::wat2wasm_with_features(wat, features).map(BinaryEncodedWasm::new)
}

#[test]
fn can_validate_valid_import_section() {
    let wasm = wat2wasm(
        r#"(module
                (import "env" "memory" (memory (;0;) 529))
                (import "env" "table" (table (;0;) 33 33 funcref))
                (import "ic0" "msg_reply" (func $reply)))"#,
    )
    .unwrap();
    assert_eq!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Ok(WasmValidationDetails {
            reserved_exports: 0,
            imports_details: WasmImportsDetails::default(),
        })
    );
}

#[test]
fn can_validate_import_section_with_invalid_memory_import() {
    let wasm = wat2wasm(r#"(module (import "foo" "memory" (memory (;0;) 529)))"#).unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidImportSection(_))
    );
}

#[test]
fn can_validate_import_section_with_invalid_table_import() {
    let wasm = wat2wasm(r#"(module (import "foo" "table" (table (;0;) 33 33 funcref)))"#).unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidImportSection(_))
    );
}

#[test]
fn can_validate_import_section_with_invalid_imported_function() {
    let wasm =
        wat2wasm(r#"(module (import "ic0" "msg_reply" (func $reply (param i64 i32))))"#).unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidFunctionSignature(_))
    );
}

#[test]
fn can_validate_valid_export_section() {
    let wasm = wat2wasm(
        r#"(module
                  (func $x)
                  (export "canister_init" (func $x))
                  (export "canister_heartbeat" (func $x))
                  (export "canister_pre_upgrade" (func $x))
                  (export "canister_post_upgrade" (func $x))
                  (export "canister_query read" (func $x)))"#,
    )
    .unwrap();

    assert_eq!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Ok(WasmValidationDetails {
            reserved_exports: 0,
            imports_details: WasmImportsDetails::default(),
        })
    );
}

#[test]
fn can_validate_valid_export_section_with_reserved_functions() {
    let wasm = wat2wasm(
        r#"(module
                  (func $x)
                  (export "canister_init" (func $x))
                  (export "canister_heartbeat" (func $x))
                  (export "canister_pre_upgrade" (func $x))
                  (export "canister_post_upgrade" (func $x))
                  (export "canister_query read" (func $x))
                  (export "some_function_is_ok" (func $x))
                  (export "canister_bar_is_reserved" (func $x))
                  (export "canister_foo_is_reserved" (func $x)))"#,
    )
    .unwrap();
    assert_eq!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Ok(WasmValidationDetails {
            reserved_exports: 2,
            imports_details: WasmImportsDetails::default(),
        })
    );
}

#[test]
fn can_validate_canister_init_with_invalid_return() {
    let wasm = wat2wasm(
        r#"(module
                  (func $x (result i32) (i32.const 0))
                  (export "canister_init" (func $x)))"#,
    )
    .unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidFunctionSignature(_))
    );
}

#[test]
fn can_validate_canister_init_with_invalid_params() {
    let wasm = wat2wasm(
        r#"(module
                  (func $x (param $y i32))
                  (export "canister_init" (func $x)))"#,
    )
    .unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidFunctionSignature(_))
    );
}

#[test]
fn can_validate_canister_heartbeat_with_invalid_return() {
    let wasm = wat2wasm(
        r#"(module
                  (func $x (result i32) (i32.const 0))
                  (export "canister_heartbeat" (func $x)))"#,
    )
    .unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidFunctionSignature(_))
    );
}

#[test]
fn can_validate_canister_heartbeat_with_invalid_params() {
    let wasm = wat2wasm(
        r#"(module
                  (func $x (param $y i32))
                  (export "canister_heartbeat" (func $x)))"#,
    )
    .unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidFunctionSignature(_))
    );
}

#[test]
fn can_validate_canister_pre_upgrade_with_invalid_return() {
    let wasm = wat2wasm(
        r#"(module
                  (func $x (result i32) (i32.const 0))
                  (export "canister_pre_upgrade" (func $x)))"#,
    )
    .unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidFunctionSignature(_))
    );
}

#[test]
fn can_validate_canister_pre_upgrade_with_invalid_params() {
    let wasm = wat2wasm(
        r#"(module
                  (func $x (param $y i32))
                  (export "canister_pre_upgrade" (func $x)))"#,
    )
    .unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidFunctionSignature(_))
    );
}

#[test]
fn can_validate_canister_post_upgrade_with_invalid_return() {
    let wasm = wat2wasm(
        r#"(module
                  (func $x (result i32) (i32.const 0))
                  (export "canister_post_upgrade" (func $x)))"#,
    )
    .unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidFunctionSignature(_))
    );
}

#[test]
fn can_validate_canister_post_upgrade_with_invalid_params() {
    let wasm = wat2wasm(
        r#"(module
                  (func $x (param $y i32))
                  (export "canister_post_upgrade" (func $x)))"#,
    )
    .unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidFunctionSignature(_))
    );
}

#[test]
fn can_validate_invalid_canister_query() {
    let wasm = wat2wasm(
        r#"(module
                    (func $read (param i64 i32) (result i32) (local.get 1))
                    (export "canister_query read" (func $read)))"#,
    )
    .unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidFunctionSignature(_))
    );
}

#[test]
fn can_validate_duplicate_method_for_canister_query_and_canister_update() {
    let wasm = wat2wasm(
        r#"(module
                    (func $read (param i64) (drop (i32.const 0)))
                    (export "canister_query read" (func $read))
                    (export "canister_update read" (func $read)))"#,
    )
    .unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidFunctionSignature(_))
    );
}

#[test]
fn can_validate_canister_query_update_method_name_with_whitespace() {
    let wasm = wat2wasm(
        r#"(module
                    (func $x)
                    (export "canister_query my_func x" (func $x))
                    (export "canister_update my_func y" (func $x)))"#,
    )
    .unwrap();
    assert_eq!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Ok(WasmValidationDetails {
            reserved_exports: 0,
            imports_details: WasmImportsDetails::default(),
        })
    );
}

#[test]
fn can_validate_valid_data_section() {
    let wasm = wat2wasm(
        r#"
                (module
                    (memory (;0;) 1)
                    (data (i32.const 0) "abcd")
                )
            "#,
    )
    .unwrap();
    assert_eq!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Ok(WasmValidationDetails {
            reserved_exports: 0,
            imports_details: WasmImportsDetails::default(),
        })
    );
}

#[test]
// this test passes currently not because of a correct validation that we're not
// using a global in data offset expression, but because we terminate the
// validation on rejecting an imported global.
fn can_validate_invalid_offset_expression_in_data_section() {
    let wasm = wat2wasm(
        r#"
                (module
                    (global (;0;) (import "test" "test") i32)
                    (memory (;0;) 1)
                    (data (global.get 0) "abcd")
                )
            "#,
    )
    .unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidImportSection(_))
    );
}

#[test]
fn can_validate_module_with_import_func() {
    // Accepts `msg_reply` from ic0 module.
    let wasm = wat2wasm(r#"(module (import "ic0" "msg_reply" (func $msg_reply)))"#).unwrap();
    assert_eq!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Ok(WasmValidationDetails {
            reserved_exports: 0,
            imports_details: WasmImportsDetails::default(),
        })
    );
}

#[test]
fn can_validate_module_with_not_allowed_import_func() {
    let wasm = wat2wasm(
        r#"(module
                    (import "msg" "my_func" (func $reply (param i32))))"#,
    )
    .unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidImportSection(_))
    );
}

#[test]
fn can_validate_module_with_wrong_import_module_for_func() {
    let wasm = wat2wasm(r#"(module (import "foo" "msg_reply" (func $reply)))"#).unwrap();
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidImportSection(_))
    );
}

#[test]
fn can_validate_module_with_too_many_globals() {
    let wasm = wat2wasm(
        r#"
                (module
                  (global (mut i32) (i32.const 0))
                  (global (mut i64) (i64.const 1))
                  (global i64 (i64.const 2))
                )
            "#,
    )
    .unwrap();
    assert_matches!(
        validate_wasm_binary(
            &wasm,
            &EmbeddersConfig {
                max_globals: 2,
                max_functions: 1024,
                ..Default::default()
            }
        ),
        Err(WasmValidationError::TooManyGlobals {
            defined: 3,
            allowed: 2
        })
    );
}

#[test]
fn can_validate_module_with_too_many_functions() {
    let wasm = wat2wasm(
        r#"
                (module
                  (func $x1)
                  (func $x2)
                  (func $x3)
                  (func $x4)
                  (func $x5)
                  (func $x6)
                )
            "#,
    )
    .unwrap();
    assert_matches!(
        validate_wasm_binary(
            &wasm,
            &EmbeddersConfig {
                max_globals: 256,
                max_functions: 5,
                ..Default::default()
            }
        ),
        Err(WasmValidationError::TooManyFunctions {
            defined: 6,
            allowed: 5
        })
    );
}

#[test]
fn can_validate_module_with_reserved_symbols() {
    for reserved_symbol in RESERVED_SYMBOLS.iter() {
        // A wasm that exports a global with a reserved name. Should fail validation.
        let wasm_global = BinaryEncodedWasm::new(
            wabt::wat2wasm(format!(
                r#"
                (module
                    (global (;0;) (mut i32) (i32.const 0))
                    (export "{}" (global 0))
                )"#,
                reserved_symbol
            ))
            .unwrap(),
        );
        assert_matches!(
            validate_wasm_binary(&wasm_global, &EmbeddersConfig::default()),
            Err(WasmValidationError::InvalidExportSection(_))
        );

        // A wasm that exports a func with a reserved name. Should fail validation.
        let wasm_func = BinaryEncodedWasm::new(
            wabt::wat2wasm(format!(
                r#"
                (module
                    (func $x)
                    (export "{}" (func $x))
                )"#,
                reserved_symbol
            ))
            .unwrap(),
        );
        assert_matches!(
            validate_wasm_binary(&wasm_func, &EmbeddersConfig::default()),
            Err(WasmValidationError::InvalidExportSection(_))
        );
    }
}

#[test]
fn can_reject_wasm_with_invalid_global_access() {
    // This wasm module defines one global but attempts to access global at index 1
    // (which would be the instruction counter after
    // instrumentation). This should
    // fail validation.
    let wasm = BinaryEncodedWasm::new(
        include_bytes!("instrumentation-test-data/invalid_global_access.wasm").to_vec(),
    );
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::WasmtimeValidation(_))
    );
}

#[test]
fn can_validate_module_with_call_simple_import() {
    // Instruments import of `call_simple` from `ic0`.
    let wasm = wat2wasm(
        r#"(module 
        (import "ic0" "call_simple" 
          (func $ic0_call_simple
            (param i32 i32)
            (param $method_name_src i32)    (param $method_name_len i32)
            (param $reply_fun i32)          (param $reply_env i32)
            (param $reject_fun i32)         (param $reject_env i32)
            (param $data_src i32)           (param $data_len i32)
            (result i32))
    ))"#,
    )
    .unwrap();

    assert_eq!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Ok(WasmValidationDetails {
            reserved_exports: 0,
            imports_details: WasmImportsDetails {
                imports_call_simple: true,
                ..Default::default()
            },
        })
    );
}

#[test]
fn can_validate_module_cycles_related_imports() {
    // Instruments imports from `ic0`.
    let wasm = wat2wasm(
        r#"(module
        (import "ic0" "call_cycles_add" (func $ic0_call_cycles_add (param $amount i64)))
        (import "ic0" "canister_cycle_balance" (func $ic0_canister_cycle_balance (result i64)))
        (import "ic0" "msg_cycles_accept" (func $ic0_msg_cycles_accept (param $amount i64) (result i64)))
    )"#,
    )
    .unwrap();

    assert_eq!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Ok(WasmValidationDetails {
            reserved_exports: 0,
            imports_details: WasmImportsDetails {
                imports_call_cycles_add: true,
                imports_canister_cycle_balance: true,
                imports_msg_cycles_accept: true,
                ..Default::default()
            },
        })
    );
}

#[test]
fn can_validate_valid_export_section_with_invalid_function_index() {
    let wasm = BinaryEncodedWasm::new(
        include_bytes!("instrumentation-test-data/export_section_invalid_function_index.wasm")
            .to_vec(),
    );
    assert_matches!(
        validate_wasm_binary(&wasm, &EmbeddersConfig::default()),
        Err(WasmValidationError::InvalidFunctionIndex {
            index: 0,
            import_count: 1
        })
    );
}

#[test]
fn can_validate_module_cycles_u128_related_imports() {
    // Instruments imports from `ic0`.
    let wasm = wat2wasm(
        r#"(module
        (import "ic0" "call_cycles_add128" (func $ic0_call_cycles_add128 (param i64 i64)))
        (import "ic0" "canister_cycles_balance128" (func $ic0_canister_cycles_balance128 (param i32)))
        (import "ic0" "msg_cycles_available128" (func $ic0_msg_cycles_available128 (param i32)))
        (import "ic0" "msg_cycles_refunded128" (func $ic0_msg_cycles_refunded128 (param i32)))
        (import "ic0" "msg_cycles_accept128" (func $ic0_msg_cycles_accept128 (param i64 i64 i32)))
    )"#,
    )
        .unwrap();

    assert_eq!(
        validate_wasm_binary(
            &wasm,
            &EmbeddersConfig {
                feature_flags: FeatureFlags {
                    api_cycles_u128_flag: FeatureStatus::Enabled
                },
                ..Default::default()
            }
        ),
        Ok(WasmValidationDetails {
            reserved_exports: 0,
            imports_details: WasmImportsDetails::default(),
        })
    );
}
