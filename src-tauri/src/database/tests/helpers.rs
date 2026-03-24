use super::*;
use crate::database::errors::DatabaseError;
use crate::database::helpers::QueryBuilder;
use std::collections::HashMap;

#[test]
fn query_builder_empty_produces_no_where_clause() {
    let qb = QueryBuilder::new();
    let (where_clause, params) = qb.build();

    assert!(
        where_clause.is_empty(),
        "expected empty string, got: {:?}",
        where_clause
    );
    assert!(params.is_empty());
}

#[test]
fn query_builder_single_condition_produces_where() {
    let mut qb = QueryBuilder::new();
    qb.add_condition("group_id = ?", 42i64);
    let (where_clause, _) = qb.build();

    assert_eq!(where_clause, "WHERE group_id = ?");
}

#[test]
fn query_builder_multiple_conditions_joined_with_and() {
    let mut qb = QueryBuilder::new();
    qb.add_condition("group_id = ?", 1i64);
    qb.add_condition("category_id = ?", 2i64);
    qb.add_condition_without_param("is_favorite = 1");
    let (where_clause, params) = qb.build();

    assert_eq!(
        where_clause,
        "WHERE group_id = ? AND category_id = ? AND is_favorite = 1"
    );
    assert_eq!(params.len(), 2);
}

#[test]
fn query_builder_paramless_condition_does_not_add_to_params() {
    let mut qb = QueryBuilder::new();
    qb.add_condition_without_param("is_favorite = 1");
    let (_, params) = qb.build();

    assert!(params.is_empty());
}

#[test]
fn validate_env_var_keys_dash_is_valid() {
    let test_db = TestDb::setup_test_db();
    let vars = Some(HashMap::from([("MY-VAR".to_string(), "value".to_string())]));
    assert!(test_db.db.validate_env_var_keys(&vars).is_ok());
}

#[test]
fn validate_env_var_keys_underscore_is_valid() {
    let test_db = TestDb::setup_test_db();

    let vars = Some(HashMap::from([("MY_VAR_123".to_string(), "v".to_string())]));
    assert!(test_db.db.validate_env_var_keys(&vars).is_ok());
}

#[test]
fn validate_env_var_keys_digit_first_is_valid() {
    let test_db = TestDb::setup_test_db();

    let vars = Some(HashMap::from([("1VAR".to_string(), "v".to_string())]));
    // The code only checks alphanumeric | '_' | '-' for every char,
    // no special first-char rule.
    assert!(test_db.db.validate_env_var_keys(&vars).is_ok());
}

#[test]
fn validate_env_var_keys_space_is_invalid() {
    let test_db = TestDb::setup_test_db();
    let vars = Some(HashMap::from([("MY VAR".to_string(), "v".to_string())]));

    let err = test_db.db.validate_env_var_keys(&vars).unwrap_err();
    assert!(matches!(
        err,
        DatabaseError::InvalidData {
            field: "env_vars",
            ..
        }
    ));
}

#[test]
fn validate_env_var_keys_dot_is_invalid() {
    let test_db = TestDb::setup_test_db();

    let vars = Some(HashMap::from([("MY.VAR".to_string(), "v".to_string())]));

    let err = test_db.db.validate_env_var_keys(&vars).unwrap_err();
    assert!(matches!(
        err,
        DatabaseError::InvalidData {
            field: "env_vars",
            ..
        }
    ));
}

#[test]
fn validate_env_var_keys_unicode_is_invalid() {
    let test_db = TestDb::setup_test_db();
    let vars = Some(HashMap::from([("VAR_🚀".to_string(), "v".to_string())]));

    let err = test_db.db.validate_env_var_keys(&vars).unwrap_err();
    assert!(matches!(
        err,
        DatabaseError::InvalidData {
            field: "env_vars",
            ..
        }
    ));
}

#[test]
fn validate_env_var_keys_none_is_ok() {
    let test_db = TestDb::setup_test_db();
    assert!(test_db.db.validate_env_var_keys(&None).is_ok());
}

#[test]
fn validate_env_var_keys_empty_map_is_ok() {
    let test_db = TestDb::setup_test_db();
    let vars: Option<HashMap<String, String>> = Some(HashMap::new());
    assert!(test_db.db.validate_env_var_keys(&vars).is_ok());
}

#[test]
fn hashmap_to_string_none_returns_ok_none() {
    let result = Database::hashmap_to_string(&None);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn hashmap_to_string_some_map_returns_json_string() {
    let map = Some(HashMap::from([("KEY".to_string(), "val".to_string())]));
    let result = Database::hashmap_to_string(&map).unwrap().unwrap();

    // Must be valid JSON
    let parsed: HashMap<String, String> = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed.get("KEY").unwrap(), "val");
}

#[test]
fn string_to_hashmap_none_returns_none() {
    let result = Database::string_to_hashmap(None);
    assert!(result.is_none());
}

#[test]
fn string_to_hashmap_invalid_json_returns_none() {
    let result = Database::string_to_hashmap(Some("not { valid json".to_string()));
    assert!(result.is_none());
}

#[test]
fn string_to_hashmap_valid_json_round_trips() {
    let json = r#"{"NODE_ENV":"production","PORT":"3000"}"#.to_string();
    let result = Database::string_to_hashmap(Some(json)).unwrap();

    assert_eq!(result.get("NODE_ENV").unwrap(), "production");
    assert_eq!(result.get("PORT").unwrap(), "3000");
}

#[test]
fn get_position_first_item_gets_position_gap() {
    let test_db = TestDb::setup_test_db();
    // "commands" table is empty for root (group_id IS NULL)
    let pos = test_db
        .db
        .get_position("commands", Some("group_id"), None)
        .unwrap();

    // COALESCE(MAX, -1) + 1 = 0; then + POSITION_GAP(1000) = 1000
    assert_eq!(pos, Database::POSITION_GAP);
}

#[test]
fn get_position_subsequent_items_increment() {
    let test_db = TestDb::setup_test_db();
    test_db.create_test_command("Test", "echo", None);

    let pos = test_db
        .db
        .get_position("commands", Some("group_id"), None)
        .unwrap();

    // After first insert (position=1000), MAX=1000, COALESCE+1=1001, +1000=2001
    assert!(pos > Database::POSITION_GAP);
}

#[test]
fn validate_field_length_whitespace_only_is_empty() {
    let test_db = TestDb::setup_test_db();
    let err = test_db
        .db
        .validate_field_length("name", "   ", Database::MAX_NAME_LENGTH)
        .unwrap_err();

    assert!(matches!(
        err,
        DatabaseError::InvalidData { field: "name", .. }
    ));
}

#[test]
fn validate_field_length_at_max_is_ok() {
    let test_db = TestDb::setup_test_db();
    let value = "a".repeat(Database::MAX_NAME_LENGTH);
    assert!(test_db
        .db
        .validate_field_length("name", &value, Database::MAX_NAME_LENGTH)
        .is_ok());
}

#[test]
fn validate_field_length_over_max_fails() {
    let test_db = TestDb::setup_test_db();
    let value = "a".repeat(Database::MAX_NAME_LENGTH + 1);
    let err = test_db
        .db
        .validate_field_length("name", &value, Database::MAX_NAME_LENGTH)
        .unwrap_err();

    assert!(matches!(
        err,
        DatabaseError::InvalidData { field: "name", .. }
    ));
}

#[test]
fn execute_db_not_found_when_id_missing() {
    let test_db = TestDb::setup_test_db();
    let err = test_db
        .db
        .execute_db(
            "commands",
            9999,
            "UPDATE commands SET name = 'x' WHERE id = ?1",
            rusqlite::params![9999i64],
        )
        .unwrap_err();

    assert!(matches!(
        err,
        DatabaseError::NotFound {
            entity: "commands",
            id: 9999
        }
    ));
}

#[test]
fn renumber_produces_gap_spaced_positions_after_exhaustion() {
    let test_db = TestDb::setup_test_db();
    let a = test_db.create_test_command("Test", "echo", None);
    let b = test_db.create_test_command("Test", "echo", None);
    let c_id = test_db.create_test_command("Test", "echo", None);

    // Exhaust the midpoint gap
    for _ in 0..15 {
        let _ = test_db.db.move_command_between(c_id, Some(a), Some(b));
    }

    let commands = test_db
        .db
        .get_commands(GroupFilter::None, CategoryFilter::All, false, None, None)
        .unwrap();

    let mut positions: Vec<i64> = commands.iter().map(|w| w.item.position).collect();
    positions.sort_unstable();

    for pair in positions.windows(2) {
        assert!(pair[1] - pair[0] >= Database::POSITION_GAP / 2);
    }
}
