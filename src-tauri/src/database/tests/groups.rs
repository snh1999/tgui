use super::*;
use crate::constants::GROUPS_TABLE;

#[test]
fn test_group_builder_pattern() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test");
    let category_id = test_db.create_test_category("Test Category");

    let mut group = GroupBuilder::new("Test")
        .with_parent(group_id)
        .with_category(category_id)
        .with_env("RUST_LOG", "debug")
        .build();

    group.shell = Some("test".to_string());
    group.position = 11;
    group.working_directory = Some("~/dir".to_string());
    let id = test_db.db.create_group(&group).unwrap();

    let retrieved_group = test_db.db.get_group(id).unwrap();
    assert_eq!(retrieved_group.name, group.name);
    assert_eq!(retrieved_group.description, group.description);
    assert_eq!(retrieved_group.working_directory, group.working_directory);
    assert_eq!(retrieved_group.env_vars, group.env_vars);
    assert_eq!(retrieved_group.shell, group.shell);
    assert_eq!(retrieved_group.category_id, group.category_id);
    assert_eq!(retrieved_group.is_favorite, false);
    assert_eq!(retrieved_group.position, Database::POSITION_GAP); // position value is auto set
}

#[test]
fn test_create_group_and_get_group() {
    let test_db = TestDb::setup_test_db();
    let group_name = "TestGroup";
    let group_id = test_db.create_test_group(group_name);

    assert!(group_id > 0);
    let group = test_db.db.get_group(group_id).unwrap();
    assert_eq!(group.name, group_name);
    assert_eq!(group.parent_group_id, None);
}

#[test]
fn test_create_group_with_all_fields() {
    let test_db = TestDb::setup_test_db();
    let parent_id = test_db.create_test_group("Parent Group");
    let category_id = test_db.create_test_category("Development");
    let env_var1 = ("var1", "value1");
    let env_var2 = ("var2", "value2");

    let mut child_group = GroupBuilder::new("Child Group")
        .with_parent(parent_id)
        .with_env(env_var1.0, env_var1.1)
        .with_env(env_var2.0, env_var2.1)
        .with_category(category_id)
        .build();

    child_group.description = Some("Run tests in release mode".to_string());
    child_group.working_directory = Some("/tmp".to_string());
    child_group.shell = Some("/bin/zsh".to_string());
    child_group.is_favorite = true;

    let child_id = test_db.save_group_to_db(&child_group);

    let retrieved_child = test_db.db.get_group(child_id).unwrap();
    let env_vars = retrieved_child.env_vars.unwrap();
    assert_eq!(env_vars.get(env_var1.0), Some(&env_var1.1.to_string()));
    assert_eq!(env_vars.get(env_var2.0), Some(&env_var2.1.to_string()));
    assert_eq!(retrieved_child.parent_group_id, Some(parent_id));
    assert_eq!(retrieved_child.category_id, Some(category_id));
    assert_eq!(
        retrieved_child.working_directory,
        child_group.working_directory
    );
    assert_eq!(retrieved_child.shell, child_group.shell);
    assert_eq!(retrieved_child.category_id, child_group.category_id);
    assert_eq!(retrieved_child.position, Database::POSITION_GAP);
    assert!(retrieved_child.is_favorite);
}

#[test]
fn test_create_group_empty_name() {
    let test_db = TestDb::setup_test_db();
    let group = GroupBuilder::new("").build();

    let result = test_db.db.create_group(&group);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
}

#[test]
fn test_create_group_whitespace_name() {
    let test_db = TestDb::setup_test_db();
    let group = GroupBuilder::new("    ").build();

    let result = test_db.db.create_group(&group);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
}

#[test]
fn test_create_group_invalid_env_var_key() {
    let test_db = TestDb::setup_test_db();
    let group = GroupBuilder::new("Test")
        .with_env("INVALID KEY!", "value")
        .build();

    let result = test_db.db.create_group(&group);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData {
            field: "env_vars",
            ..
        })
    ));
}

#[test]
fn test_create_group_circular_reference_direct() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test");

    let mut group = test_db.db.get_group(group_id).unwrap();
    group.parent_group_id = Some(group_id); // Self-reference

    let result = test_db.db.update_group(&group);
    assert!(matches!(
        result,
        Err(DatabaseError::CircularReference { .. })
    ));
}

#[test]
fn test_create_group_circular_reference_indirect() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Root");

    let child_group = GroupBuilder::new("Child").with_parent(root_id).build();
    let child_id = test_db.save_group_to_db(&child_group);

    // Try to make root a child of child (creates cycle)
    let mut root = test_db.db.get_group(root_id).unwrap();
    root.parent_group_id = Some(child_id);

    let result = test_db.db.update_group(&root);
    assert!(matches!(
        result,
        Err(DatabaseError::CircularReference { .. })
    ));
}

#[test]
fn test_get_group_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.get_group(99999);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: GROUPS_TABLE,
            id: 99999
        })
    ));
}

#[test]
fn test_get_groups_by_parent() {
    let test_db = TestDb::setup_test_db();
    let parent_id = test_db.create_test_group("Parent");

    let child_1 = GroupBuilder::new("Child1").with_parent(parent_id).build();
    test_db.save_group_to_db(&child_1);

    let child_2 = GroupBuilder::new("Child1").with_parent(parent_id).build();
    test_db.save_group_to_db(&child_2);

    test_db.create_test_group("Orphan");

    let children = test_db.db.get_groups(Some(parent_id), None, false).unwrap();
    assert_eq!(children.len(), 2);
    assert!(children
        .iter()
        .all(|g| g.parent_group_id == Some(parent_id)));
}

#[test]
fn test_get_root_groups() {
    let test_db = TestDb::setup_test_db();
    test_db.create_test_group("Root1");
    test_db.create_test_group("Root2");

    let parent = test_db.create_test_group("Parent");
    let child = GroupBuilder::new("Child").with_parent(parent).build();
    test_db.save_group_to_db(&child);

    let roots = test_db.db.get_groups(None, None, false).unwrap();
    assert_eq!(roots.len(), 3); // Root1, Root2, Parent
}

#[test]
fn test_get_groups_by_category() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let category_id = test_db.create_test_category("Test Category");

    test_db.create_test_category("One");
    test_db.create_test_category("Two");
    test_db.save_group_to_db(
        &GroupBuilder::new("cmd3")
            .with_parent(group_id)
            .with_category(category_id)
            .build(),
    );

    let commands = test_db
        .db
        .get_groups(Some(group_id), Some(category_id), false)
        .unwrap();
    assert_eq!(commands.len(), 1);
    assert!(commands.iter().all(|c| c.category_id == Some(category_id)));
}

#[test]
fn test_get_favorite_groups() {
    let test_db = TestDb::setup_test_db();
    let id1 = test_db.create_test_group("Group1");
    let id2 = test_db.create_test_group("Group2");
    test_db.create_test_group("Group3");

    test_db.db.toggle_group_favorite(id1).unwrap();
    test_db.db.toggle_group_favorite(id2).unwrap();

    let favorites = test_db.db.get_groups(None, None, true).unwrap();
    assert_eq!(favorites.len(), 2);
}

#[test]
fn test_toggle_favorite_group() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let initial = test_db.db.get_group(group_id).unwrap().is_favorite;
    test_db.db.toggle_group_favorite(group_id).unwrap();

    assert_eq!(
        !initial,
        test_db.db.get_group(group_id).unwrap().is_favorite
    );
}

#[test]
fn test_toggle_favorite_group_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.toggle_group_favorite(99999);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: GROUPS_TABLE,
            id: 99999
        })
    ));
}

#[test]
fn test_get_favorite_group_with_parent() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let group_id2 = test_db.create_test_group("Group1");

    let mut fav_group = GroupBuilder::new("Child").with_parent(group_id).build();
    fav_group.is_favorite = true;
    let fav_id = test_db.save_group_to_db(&fav_group);

    let non_fav_group = GroupBuilder::new("Not fav").with_parent(group_id).build();
    test_db.save_group_to_db(&non_fav_group);

    let mut other_parent_group = GroupBuilder::new("Not").with_parent(group_id2).build();
    other_parent_group.is_favorite = true;
    test_db.save_group_to_db(&other_parent_group);

    let favorites = test_db.db.get_groups(Some(group_id), None, true).unwrap();

    assert_eq!(favorites.len(), 1);
    assert_eq!(favorites[0].id, fav_id);
}

#[test]
fn test_update_group() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Original");

    let mut group = test_db.db.get_group(group_id).unwrap();
    let updated_name = "Updated";
    let updated_description = "Updated description";
    group.name = updated_name.to_string();
    group.description = Some(updated_description.to_string());

    test_db.db.update_group(&group).unwrap();

    let updated = test_db.db.get_group(group_id).unwrap();
    assert_eq!(updated.name, updated_name.to_string());
    assert_eq!(updated.description, Some(updated_description.to_string()));
}

#[test]
fn test_update_group_not_found() {
    let test_db = TestDb::setup_test_db();
    let group = GroupBuilder::new("Ghost").build();
    let result = test_db.db.update_group(&group);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: GROUPS_TABLE,
            id: 0
        })
    ));
}

#[test]
fn test_update_group_position() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test");
    let original_pos = test_db.db.get_group(group_id).unwrap().position;

    let mut group = test_db.db.get_group(group_id).unwrap();
    group.position = 0;
    test_db.db.update_group(&group).unwrap();

    let updated = test_db.db.get_group(group_id).unwrap();
    assert_eq!(updated.position, original_pos); // Position should not change on update
}

#[test]
fn test_update_group_validation() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let mut group = test_db.db.get_group(group_id).unwrap();
    group.name = "".to_string();

    let result = test_db.db.update_group(&group);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
}

#[test]
fn test_delete_group_cascades_to_commands() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Deletable");
    test_db.create_test_command("Orphaned", "echo test", Some(group_id));
    test_db.create_test_command("Child2", "echo child2", Some(group_id));

    test_db.db.delete_group(group_id).unwrap();

    let commands = test_db
        .db
        .get_commands(Some(group_id), None, false)
        .unwrap();
    assert_eq!(commands.len(), 0);
}

#[test]
fn test_delete_group_cascades_to_child_groups() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Deletable");
    test_db.save_group_to_db(
        &GroupBuilder::new("first child")
            .with_parent(group_id)
            .build(),
    );
    test_db.save_group_to_db(
        &GroupBuilder::new("second child")
            .with_parent(group_id)
            .build(),
    );

    test_db.db.delete_group(group_id).unwrap();

    let commands = test_db.db.get_groups(Some(group_id), None, false).unwrap();
    assert_eq!(commands.len(), 0);
}

#[test]
fn test_delete_group_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.delete_group(99999);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: GROUPS_TABLE,
            id: 99999
        })
    ));
}

#[test]
fn test_get_group_tree() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Project");

    let api_id = test_db.save_group_to_db(&GroupBuilder::new("API").with_parent(root_id).build());
    let auth_id = test_db.save_group_to_db(&GroupBuilder::new("Auth").with_parent(root_id).build());
    let web_id = test_db.save_group_to_db(&GroupBuilder::new("Web").with_parent(root_id).build());

    let tree = test_db.db.get_group_tree(root_id).unwrap();
    assert_eq!(tree.len(), 4); // All descendants + root

    // Should include all levels
    let ids: Vec<i64> = tree.iter().map(|g| g.id.clone()).collect();
    assert!(ids.contains(&root_id));
    assert!(ids.contains(&auth_id));
    assert!(ids.contains(&api_id));
    assert!(ids.contains(&web_id));
}

#[test]
fn test_get_group_path() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Project");
    let api_id = test_db.save_group_to_db(&GroupBuilder::new("API").with_parent(root_id).build());
    let auth_id = test_db.save_group_to_db(&GroupBuilder::new("Auth").with_parent(api_id).build());

    let path = test_db.db.get_group_path(auth_id).unwrap();
    assert_eq!(path, vec!["Project", "API", "Auth"]);
}

#[test]
fn test_get_group_path_root_level() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Root");

    let path = test_db.db.get_group_path(root_id).unwrap();
    assert_eq!(path, vec!["Root"]);
}

#[test]
fn test_get_group_command_count() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test");

    test_db.create_test_command("Cmd1", "echo 1", Some(group_id));
    test_db.create_test_command("Cmd2", "echo 2", Some(group_id));
    test_db.create_test_command("Standalone", "echo 3", None);

    let count = test_db.db.get_group_command_count(group_id).unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_move_group_to_end() {
    let test_db = TestDb::setup_test_db();
    let parent_id = test_db.create_test_group("Parent");

    let id3 = test_db.save_group_to_db(&GroupBuilder::new("Z").with_parent(parent_id).build());
    let id1 = test_db.save_group_to_db(&GroupBuilder::new("A").with_parent(parent_id).build());
    let id2 = test_db.save_group_to_db(&GroupBuilder::new("M").with_parent(parent_id).build());

    test_db.db.move_group_between(id3, Some(id2), None).unwrap();

    let children = test_db.db.get_groups(Some(parent_id), None, false).unwrap();
    assert_eq!(children[0].id, id1);
    assert_eq!(children[1].id, id2);
    assert_eq!(children[2].id, id3);
}

#[test]
fn test_move_group_to_top() {
    let test_db = TestDb::setup_test_db();
    let parent_id = test_db.create_test_group("Parent");

    let id3 = test_db.save_group_to_db(&GroupBuilder::new("Z").with_parent(parent_id).build());
    let id1 = test_db.save_group_to_db(&GroupBuilder::new("A").with_parent(parent_id).build());
    let id2 = test_db.save_group_to_db(&GroupBuilder::new("M").with_parent(parent_id).build());

    test_db.db.move_group_between(id1, None, Some(id3)).unwrap();

    let children = test_db.db.get_groups(Some(parent_id), None, false).unwrap();
    assert_eq!(children[0].id, id1);
    assert_eq!(children[1].id, id3);
    assert_eq!(children[2].id, id2);
}

#[test]
fn test_move_group_to_middle() {
    let test_db = TestDb::setup_test_db();
    let parent_id = test_db.create_test_group("Parent");

    let id1 = test_db.save_group_to_db(&GroupBuilder::new("A").with_parent(parent_id).build());
    let id2 = test_db.save_group_to_db(&GroupBuilder::new("M").with_parent(parent_id).build());

    let id3 = test_db.save_group_to_db(&GroupBuilder::new("Z").with_parent(parent_id).build());

    test_db
        .db
        .move_group_between(id3, Some(id1), Some(id2))
        .unwrap();

    let children = test_db.db.get_groups(Some(parent_id), None, false).unwrap();
    assert_eq!(children[0].id, id1);
    assert_eq!(children[1].id, id3);
    assert_eq!(children[2].id, id2);
}

#[test]
fn test_updated_at_changes_on_update() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test");

    let group = test_db.db.get_group(group_id).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1000)); // Ensure time difference

    let mut group = group.clone();
    group.name = "Updated".to_string();
    test_db.db.update_group(&group).unwrap();

    let retrieved_group = test_db.db.get_group(group_id).unwrap();
    assert_ne!(group.updated_at, retrieved_group.updated_at);
    assert_eq!(group.created_at, retrieved_group.created_at);
}
