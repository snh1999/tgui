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
    group.working_directory = Some(TestDb::get_temp_dir());
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
fn test_get_groups_empty() {
    let test_db = TestDb::setup_test_db();
    let groups = test_db
        .db
        .get_groups(GroupFilter::All, CategoryFilter::All, false)
        .unwrap();
    assert!(groups.is_empty());
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
fn test_create_group_name_max_length() {
    let test_db = TestDb::setup_test_db();
    let valid_name = "a".repeat(Database::MAX_NAME_LENGTH);
    let group = GroupBuilder::new(&valid_name).build();
    assert!(test_db.db.create_group(&group).is_ok());

    let invalid_name = "a".repeat(Database::MAX_NAME_LENGTH + 1);
    let group = GroupBuilder::new(&invalid_name).build();
    let result = test_db.db.create_group(&group);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
}

#[test]
fn test_create_group_whitespace_variations_rejected() {
    let test_db = TestDb::setup_test_db();

    for ws in ["\t", "\n", "  \t  \n  "] {
        let group = GroupBuilder::new(ws).build();
        let result = test_db.db.create_group(&group);
        assert!(
            matches!(
                result,
                Err(DatabaseError::InvalidData { field: "name", .. })
            ),
            "Failed for whitespace: {:?}",
            ws
        );
    }
}

#[test]
fn test_create_group_env_var_edge_cases() {
    let test_db = TestDb::setup_test_db();

    let group = GroupBuilder::new("Test")
        .with_env("MY-VAR", "value")
        .build();
    assert!(test_db.db.create_group(&group).is_ok());

    let group = GroupBuilder::new("Test2").with_env("VAR_123", "v").build();
    assert!(test_db.db.create_group(&group).is_ok());

    let group = GroupBuilder::new("Test3").with_env("BAD KEY", "v").build();
    assert!(matches!(
        test_db.db.create_group(&group),
        Err(DatabaseError::InvalidData {
            field: "env_vars",
            ..
        })
    ));
}

#[test]
fn test_create_group_with_nonexistent_parent_fails() {
    let test_db = TestDb::setup_test_db();
    let group = GroupBuilder::new("Orphan").with_parent(99).build();
    let result = test_db.db.create_group(&group);
    assert!(matches!(
        result,
        Err(DatabaseError::ForeignKeyViolation { .. })
    ));
}

#[test]
fn test_create_group_with_nonexistent_category_fails() {
    let test_db = TestDb::setup_test_db();
    let group = GroupBuilder::new("Test").with_category(999).build();
    let result = test_db.db.create_group(&group);
    assert!(matches!(
        result,
        Err(DatabaseError::ForeignKeyViolation { .. })
    ));
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

    let children = test_db
        .db
        .get_groups(GroupFilter::Group(parent_id), CategoryFilter::None, false)
        .unwrap();
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

    let roots = test_db
        .db
        .get_groups(GroupFilter::None, CategoryFilter::None, false)
        .unwrap();
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
        .get_groups(
            GroupFilter::Group(group_id),
            CategoryFilter::Category(category_id),
            false,
        )
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

    let favorites = test_db
        .db
        .get_groups(GroupFilter::None, CategoryFilter::None, true)
        .unwrap();
    assert_eq!(favorites.len(), 2);
}

#[test]
fn test_get_groups_filter_none_returns_only_root_groups() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Root");
    let child = GroupBuilder::new("Child").with_parent(root_id).build();
    test_db.db.create_group(&child).unwrap();
    test_db.db.create_group(&child).unwrap();

    let groups = test_db
        .db
        .get_groups(GroupFilter::None, CategoryFilter::All, false)
        .unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].id, root_id);
}

#[test]
fn test_get_groups_filter_group_returns_only_children_of_that_parent() {
    let test_db = TestDb::setup_test_db();
    let parent_a = test_db.create_test_group("A");
    let parent_b = test_db.create_test_group("B");

    let child_a = GroupBuilder::new("ChildA").with_parent(parent_a).build();
    let child_a_id = test_db.db.create_group(&child_a).unwrap();

    let child_b = GroupBuilder::new("ChildB").with_parent(parent_b).build();
    test_db.db.create_group(&child_b).unwrap();

    let groups = test_db
        .db
        .get_groups(GroupFilter::Group(parent_a), CategoryFilter::All, false)
        .unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].id, child_a_id);
}

#[test]
fn test_get_groups_filter_all_returns_every_group() {
    let test_db = TestDb::setup_test_db();
    let root_id_1 = test_db.create_test_group("Root");
    let root_id_2 = test_db.create_test_group("Root");
    let child = GroupBuilder::new("Child").with_parent(root_id_1).build();
    test_db.db.create_group(&child).unwrap();

    let child = GroupBuilder::new("Child").with_parent(root_id_2).build();
    test_db.db.create_group(&child).unwrap();

    let groups = test_db
        .db
        .get_groups(GroupFilter::All, CategoryFilter::All, false)
        .unwrap();

    assert_eq!(groups.len(), 4);
}

#[test]
fn test_get_groups_category_filter_none_excludes_categorized_groups() {
    let test_db = TestDb::setup_test_db();
    let cat_id = test_db.create_test_category("Dev");

    test_db.create_test_group("NoCat");
    let category = GroupBuilder::new("WithCat").with_category(cat_id).build();
    test_db.db.create_group(&category).unwrap();

    let groups = test_db
        .db
        .get_groups(GroupFilter::All, CategoryFilter::None, false)
        .unwrap();

    assert_eq!(groups.len(), 1);
    assert!(groups[0].category_id.is_none());
}

#[test]
fn test_get_groups_category_filter_category_returns_matching_only() {
    let test_db = TestDb::setup_test_db();
    let cat_id = test_db.create_test_category("Dev");

    test_db.create_test_group("No Category");
    test_db.create_test_group("Uncategorized");
    let with_cat = GroupBuilder::new("WithCat").with_category(cat_id).build();
    let cat_group_id = test_db.db.create_group(&with_cat).unwrap();

    let groups = test_db
        .db
        .get_groups(GroupFilter::All, CategoryFilter::Category(cat_id), false)
        .unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].id, cat_group_id);
}

#[test]
fn test_get_groups_ordered_by_position() {
    let test_db = TestDb::setup_test_db();
    let id_1 = test_db.create_test_group("First");
    let id_2 = test_db.create_test_group("Second");
    let id_3 = test_db.create_test_group("First");

    let groups = test_db
        .db
        .get_groups(GroupFilter::All, CategoryFilter::All, false)
        .unwrap();

    assert_eq!(groups[0].id, id_1);
    assert_eq!(groups[1].id, id_2);
    assert_eq!(groups[2].id, id_3);
}

#[test]
fn test_get_groups_count_zero() {
    let test_db = TestDb::setup_test_db();
    let count = test_db.db.get_groups_count(None, None, false).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_get_groups_count_root_only() {
    let test_db = TestDb::setup_test_db();
    test_db.create_test_group("A");
    test_db.create_test_group("B");
    let count = test_db.db.get_groups_count(None, None, false).unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_get_groups_count_favorites_only() {
    let test_db = TestDb::setup_test_db();
    test_db.create_test_group("Regular");
    test_db.create_test_group("Test");
    let fav_id = test_db.create_test_group("Favorite");
    test_db.db.toggle_group_favorite(fav_id).unwrap();
    let fav_id = test_db.create_test_group("Favorite");
    test_db.db.toggle_group_favorite(fav_id).unwrap();

    let count = test_db.db.get_groups_count(None, None, true).unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_get_groups_count_by_category() {
    let test_db = TestDb::setup_test_db();
    let cat_id = test_db.create_test_category("Dev");

    test_db.create_test_group("No Category");
    let with_cat = GroupBuilder::new("With Cat").with_category(cat_id).build();
    test_db.db.create_group(&with_cat).unwrap();
    test_db.db.create_group(&with_cat).unwrap();

    let count = test_db
        .db
        .get_groups_count(None, Some(cat_id), false)
        .unwrap();
    assert_eq!(count, 2);
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

    test_db.db.toggle_group_favorite(group_id).unwrap();
    assert_eq!(initial, test_db.db.get_group(group_id).unwrap().is_favorite);
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

    let favorites = test_db
        .db
        .get_groups(GroupFilter::Group(group_id), CategoryFilter::None, true)
        .unwrap();

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
fn test_update_group_clears_optional_fields() {
    let test_db = TestDb::setup_test_db();
    let mut group = GroupBuilder::new("Dev").build();
    group.shell = Some("/bin/bash".to_string());
    group.description = Some("Test Description".to_string());
    let id = test_db.db.create_group(&group).unwrap();

    let mut updated = test_db.db.get_group(id).unwrap();
    updated.description = None;
    updated.shell = None;
    test_db.db.update_group(&updated).unwrap();

    let fetched = test_db.db.get_group(id).unwrap();
    assert_eq!(fetched.description, None);
    assert_eq!(fetched.shell, None);
}

#[test]
fn test_update_group_empty_name_fails() {
    let test_db = TestDb::setup_test_db();
    let id = test_db.create_test_group("Valid");

    let mut group = test_db.db.get_group(id).unwrap();
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
        .get_commands(
            GroupFilter::Group(group_id),
            CategoryFilter::None,
            false,
            None,
            None,
        )
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

    let commands = test_db
        .db
        .get_groups(GroupFilter::Group(group_id), CategoryFilter::None, false)
        .unwrap();
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
fn test_get_group_tree_root_only() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Root");

    let tree = test_db.db.get_group_tree(root_id).unwrap();

    assert_eq!(tree.group.id, root_id);
    assert!(tree.children.is_empty());
}

#[test]
fn test_get_group_tree() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Project");

    let api_id = test_db.save_group_to_db(&GroupBuilder::new("API").with_parent(root_id).build());
    let auth_id = test_db.save_group_to_db(&GroupBuilder::new("Auth").with_parent(root_id).build());
    let web_id = test_db.save_group_to_db(&GroupBuilder::new("Web").with_parent(root_id).build());

    let tree = test_db.db.get_group_tree(root_id).unwrap();

    assert_eq!(tree.group.id, root_id);
    assert_eq!(tree.children.len(), 3);

    let child_ids: Vec<i64> = tree.children.iter().map(|n| n.group.id).collect();
    assert!(child_ids.contains(&api_id));
    assert!(child_ids.contains(&auth_id));
    assert!(child_ids.contains(&web_id));
}

#[test]
fn tree_includes_multiple_levels_of_nesting() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Root");
    let child_id =
        test_db.save_group_to_db(&GroupBuilder::new("Child").with_parent(root_id).build());
    let grandchild_id = test_db.save_group_to_db(
        &GroupBuilder::new("Grandchild")
            .with_parent(child_id)
            .build(),
    );
    let great_id = test_db.save_group_to_db(
        &GroupBuilder::new("Great")
            .with_parent(grandchild_id)
            .build(),
    );

    let tree = test_db.db.get_group_tree(root_id).unwrap();

    assert_eq!(tree.group.id, root_id);
    let child = &tree.children[0];
    assert_eq!(child.group.id, child_id);
    let grandchild = &child.children[0];
    assert_eq!(grandchild.group.id, grandchild_id);
    assert_eq!(grandchild.children[0].group.id, great_id);
}

#[test]
fn tree_includes_all_siblings_and_preserves_order() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Root");

    let branch1_id =
        test_db.save_group_to_db(&GroupBuilder::new("Branch1").with_parent(root_id).build());
    let branch2_id =
        test_db.save_group_to_db(&GroupBuilder::new("Branch2").with_parent(root_id).build());

    let b1_child1_id = test_db.save_group_to_db(
        &GroupBuilder::new("B1Child1")
            .with_parent(branch1_id)
            .build(),
    );
    let b1_child2_id = test_db.save_group_to_db(
        &GroupBuilder::new("B1Child2")
            .with_parent(branch1_id)
            .build(),
    );
    let b2_child1_id = test_db.save_group_to_db(
        &GroupBuilder::new("B2Child1")
            .with_parent(branch2_id)
            .build(),
    );

    let tree = test_db.db.get_group_tree(root_id).unwrap();

    assert_eq!(tree.group.id, root_id);
    assert_eq!(tree.children.len(), 2);

    let branch1 = &tree.children[0];
    let branch2 = &tree.children[1];
    assert_eq!(branch1.group.id, branch1_id);
    assert_eq!(branch2.group.id, branch2_id);

    assert_eq!(branch1.children.len(), 2);
    assert_eq!(branch1.children[0].group.id, b1_child1_id);
    assert_eq!(branch1.children[1].group.id, b1_child2_id);

    assert_eq!(branch2.children.len(), 1);
    assert_eq!(branch2.children[0].group.id, b2_child1_id);
}

#[test]
fn tree_of_leaf_node_returns_only_itself() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Root");
    let leaf_id = test_db.save_group_to_db(&GroupBuilder::new("Leaf").with_parent(root_id).build());

    let tree = test_db.db.get_group_tree(leaf_id).unwrap();

    assert_eq!(tree.group.id, leaf_id);
    assert!(tree.children.is_empty());
}

#[test]
fn tree_does_not_include_unrelated_groups() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Root");
    let child_id =
        test_db.save_group_to_db(&GroupBuilder::new("Child").with_parent(root_id).build());
    test_db.create_test_group("Unrelated");

    let tree = test_db.db.get_group_tree(root_id).unwrap();

    assert_eq!(tree.children.len(), 1);
    assert_eq!(tree.children[0].group.id, child_id);
}

#[test]
fn tree_nonexistent_root_returns_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.get_group_tree(99);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: GROUPS_TABLE,
            id: 99
        })
    ));
}

#[test]
fn ancestor_chain_returns_direct_parent_first() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Root");
    let child_id =
        test_db.save_group_to_db(&GroupBuilder::new("Child").with_parent(root_id).build());
    let grandchild_id = test_db.save_group_to_db(
        &GroupBuilder::new("Grandchild")
            .with_parent(child_id)
            .build(),
    );

    let chain = test_db.db.get_group_ancestor_chain(grandchild_id).unwrap();
    assert_eq!(
        chain[0].id, grandchild_id,
        "First entry should be the group itself"
    );
    assert_eq!(chain[1].id, child_id, "Second should be direct parent");
    assert_eq!(chain[2].id, root_id, "Third should be root");
}

#[test]
fn ancestor_chain_of_root_returns_only_itself() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Root");

    let chain = test_db.db.get_group_ancestor_chain(root_id).unwrap();
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].id, root_id);
}

#[test]
fn ancestor_chain_does_not_include_siblings() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Root");
    let child_id =
        test_db.save_group_to_db(&GroupBuilder::new("Child").with_parent(root_id).build());
    let sibling_id =
        test_db.save_group_to_db(&GroupBuilder::new("Sibling").with_parent(root_id).build());

    let chain = test_db.db.get_group_ancestor_chain(child_id).unwrap();
    let ids: Vec<i64> = chain.iter().map(|g| g.id).collect();

    assert!(
        !ids.contains(&sibling_id),
        "Sibling should not appear in ancestor chain"
    );
}

#[test]
fn ancestor_chain_of_sibling_shares_same_ancestors_but_not_each_other() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Root");
    let parent_id =
        test_db.save_group_to_db(&GroupBuilder::new("Parent").with_parent(root_id).build());
    let sib1_id =
        test_db.save_group_to_db(&GroupBuilder::new("Sib1").with_parent(parent_id).build());
    let sib2_id =
        test_db.save_group_to_db(&GroupBuilder::new("Sib2").with_parent(parent_id).build());

    let chain1: Vec<i64> = test_db
        .db
        .get_group_ancestor_chain(sib1_id)
        .unwrap()
        .iter()
        .map(|g| g.id)
        .collect();
    let chain2: Vec<i64> = test_db
        .db
        .get_group_ancestor_chain(sib2_id)
        .unwrap()
        .iter()
        .map(|g| g.id)
        .collect();

    // Exact order: self → parent → root
    assert_eq!(chain1, vec![sib1_id, parent_id, root_id]);
    assert_eq!(chain2, vec![sib2_id, parent_id, root_id]);
}

#[test]
fn ancestor_chain_nonexistent_id_returns_empty() {
    let test_db = TestDb::setup_test_db();
    let chain = test_db.db.get_group_ancestor_chain(999_999).unwrap();
    assert!(chain.is_empty());
}

#[test]
fn ancestor_chain_length_matches_depth() {
    let test_db = TestDb::setup_test_db();
    let root_id = test_db.create_test_group("Root");
    let l1 = test_db.save_group_to_db(&GroupBuilder::new("L1").with_parent(root_id).build());
    let l2 = test_db.save_group_to_db(&GroupBuilder::new("L2").with_parent(l1).build());
    let l3 = test_db.save_group_to_db(&GroupBuilder::new("L3").with_parent(l2).build());

    let chain = test_db.db.get_group_ancestor_chain(l3).unwrap();
    assert_eq!(chain.len(), 4, "Should include l3 + l2 + l1 + root");
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
fn test_group_path_nonexistent_id() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.get_group_path(99999).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_move_group_to_end() {
    let test_db = TestDb::setup_test_db();
    let parent_id = test_db.create_test_group("Parent");

    let id3 = test_db.save_group_to_db(&GroupBuilder::new("Z").with_parent(parent_id).build());
    let id1 = test_db.save_group_to_db(&GroupBuilder::new("A").with_parent(parent_id).build());
    let id2 = test_db.save_group_to_db(&GroupBuilder::new("M").with_parent(parent_id).build());

    test_db.db.move_group_between(id3, Some(id2), None).unwrap();

    let children = test_db
        .db
        .get_groups(GroupFilter::Group(parent_id), CategoryFilter::None, false)
        .unwrap();
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

    let children = test_db
        .db
        .get_groups(GroupFilter::Group(parent_id), CategoryFilter::None, false)
        .unwrap();
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

    let children = test_db
        .db
        .get_groups(GroupFilter::Group(parent_id), CategoryFilter::None, false)
        .unwrap();
    assert_eq!(children[0].id, id1);
    assert_eq!(children[1].id, id3);
    assert_eq!(children[2].id, id2);
}

#[test]
fn test_move_group_between_both_none_returns_error() {
    let test_db = TestDb::setup_test_db();
    let id = test_db.create_test_group("G");

    let result = test_db.db.move_group_between(id, None, None);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData {
            field: "item_id",
            ..
        })
    ));
}

#[test]
fn test_move_group_between_cross_parent_returns_error() {
    let test_db = TestDb::setup_test_db();
    let parent_a = test_db.create_test_group("ParentA");
    let parent_b = test_db.create_test_group("ParentB");

    let g = GroupBuilder::new("G").with_parent(parent_a).build();
    let g_id = test_db.db.create_group(&g).unwrap();
    let prev = GroupBuilder::new("Prev").with_parent(parent_a).build();
    let prev_id = test_db.db.create_group(&prev).unwrap();
    let next = GroupBuilder::new("Next").with_parent(parent_b).build();
    let next_id = test_db.db.create_group(&next).unwrap();

    let result = test_db
        .db
        .move_group_between(g_id, Some(prev_id), Some(next_id));
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData {
            field: "parent_id",
            ..
        })
    ));
}

#[test]
fn test_move_group_between_invalid_prev_id() {
    let test_db = TestDb::setup_test_db();
    let parent_id = test_db.create_test_group("Parent");
    let group_id = test_db.save_group_to_db(&GroupBuilder::new("G").with_parent(parent_id).build());
    let real_sibling =
        test_db.save_group_to_db(&GroupBuilder::new("S").with_parent(parent_id).build());

    let result = test_db
        .db
        .move_group_between(group_id, Some(999), Some(real_sibling));
    assert!(matches!(result, Err(DatabaseError::NotFound { .. })));
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

#[test]
fn test_update_group_self_reference_is_circular() {
    let test_db = TestDb::setup_test_db();
    let id = test_db.create_test_group("Lone");

    let mut group = test_db.db.get_group(id).unwrap();
    group.parent_group_id = Some(id);

    let result = test_db.db.update_group(&group);
    assert!(matches!(
        result,
        Err(DatabaseError::CircularReference { .. })
    ));
}

#[test]
fn test_update_group_cycle_detected() {
    let test_db = TestDb::setup_test_db();
    let a_id = test_db.create_test_group("A");
    let b = GroupBuilder::new("B").with_parent(a_id).build();
    let b_id = test_db.db.create_group(&b).unwrap();
    let c = GroupBuilder::new("C").with_parent(b_id).build();
    let c_id = test_db.db.create_group(&c).unwrap();

    let mut group_a = test_db.db.get_group(a_id).unwrap();
    group_a.parent_group_id = Some(c_id);

    let result = test_db.db.update_group(&group_a);
    assert!(matches!(
        result,
        Err(DatabaseError::CircularReference { .. })
    ));
}

#[test]
fn test_update_group_valid_reparenting_succeeds() {
    let test_db = TestDb::setup_test_db();
    let old_parent = test_db.create_test_group("OldParent");
    let new_parent = test_db.create_test_group("NewParent");
    let child = GroupBuilder::new("Child").with_parent(old_parent).build();
    let child_id = test_db.db.create_group(&child).unwrap();

    let mut group = test_db.db.get_group(child_id).unwrap();
    group.parent_group_id = Some(new_parent);
    test_db.db.update_group(&group).unwrap();

    let fetched = test_db.db.get_group(child_id).unwrap();
    assert_eq!(fetched.parent_group_id, Some(new_parent));
}

#[test]
fn test_deep_hierarchy_circular_reference() {
    let test_db = TestDb::setup_test_db();
    let mut current_id = test_db.create_test_group("Level0");

    // Create a 50-level deep hierarchy
    for i in 1..=50 {
        current_id = test_db.save_group_to_db(
            &GroupBuilder::new(&format!("Level{}", i))
                .with_parent(current_id)
                .build(),
        );
    }

    // Try to make Level0 a child of Level50 (should fail)
    let mut root = test_db
        .db
        .get_group(
            test_db
                .db
                .get_group(current_id)
                .unwrap()
                .parent_group_id
                .unwrap(),
        )
        .unwrap();
    root.parent_group_id = Some(current_id);

    let result = test_db.db.update_group(&root);
    assert!(matches!(
        result,
        Err(DatabaseError::CircularReference { .. })
    ));
}

#[test]
fn test_update_group_with_empty_description() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test");

    let mut group = test_db.db.get_group(group_id).unwrap();
    group.description = Some("Has description".to_string());
    test_db.db.update_group(&group).unwrap();

    let mut group = test_db.db.get_group(group_id).unwrap();
    group.description = None;
    test_db.db.update_group(&group).unwrap();

    let retrieved = test_db.db.get_group(group_id).unwrap();
    assert_eq!(retrieved.description, None);
}

#[test]
fn test_update_group_clear_working_directory() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test");

    let mut group = test_db.db.get_group(group_id).unwrap();
    group.working_directory = Some("/tmp".to_string());
    test_db.db.update_group(&group).unwrap();

    let mut group = test_db.db.get_group(group_id).unwrap();
    group.working_directory = None;
    test_db.db.update_group(&group).unwrap();

    let retrieved = test_db.db.get_group(group_id).unwrap();
    assert_eq!(retrieved.working_directory, None);
}

#[test]
fn test_search_groups_by_name() {
    let test_db = TestDb::setup_test_db();
    test_db.create_test_group("Frontend App");
    test_db.create_test_group("Backend API");
    test_db.create_test_group("Frontend Admin");
    test_db.create_test_group("Database");

    let results = test_db.db.search_groups("frontend").unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_groups_matches_description() {
    let test_db = TestDb::setup_test_db();
    let mut group = GroupBuilder::new("Misc").build();
    group.description = Some("all the related tasks".to_string());
    test_db.db.create_group(&group).unwrap();

    group.description = Some("related".to_string());
    test_db.db.create_group(&group).unwrap();

    group.description = Some("no relation".to_string());
    test_db.db.create_group(&group).unwrap();

    test_db.create_test_group("Other");

    let results = test_db.db.search_groups("related").unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_groups_empty_term_returns_all() {
    let test_db = TestDb::setup_test_db();
    test_db.create_test_group("Alpha");
    test_db.create_test_group("Beta");

    let results = test_db.db.search_groups("").unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_groups_sql_wildcard_behavior() {
    let test_db = TestDb::setup_test_db();

    test_db.create_test_group("Test_Group");
    test_db.create_test_group("Test%2");
    test_db.create_test_group("TestXGroup");

    // % matches any sequence in SQLite LIKE
    let results = test_db.db.search_groups("Test%").unwrap();
    assert_eq!(results.len(), 3);

    let results = test_db.db.search_groups("Test_").unwrap();
    assert_eq!(results.len(), 3);

    // _ matches single char
    let results = test_db.db.search_groups("Test_Group").unwrap();
    assert!(results.len() >= 2);
}

#[test]
fn test_search_groups_by_description() {
    let test_db = TestDb::setup_test_db();
    let mut g1 = GroupBuilder::new("Proj1").build();
    g1.description = Some("Web application project".to_string());
    test_db.save_group_to_db(&g1);

    let mut g2 = GroupBuilder::new("Proj2").build();
    g2.description = Some("Mobile app project".to_string());
    test_db.save_group_to_db(&g2);

    let results = test_db.db.search_groups("project").unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_groups_none_found() {
    let test_db = TestDb::setup_test_db();
    test_db.create_test_group("Alpha");
    test_db.create_test_group("Beta");

    let results = test_db.db.search_groups("nonexistent").unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_update_group_icon_and_color() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test");

    let mut group = test_db.db.get_group(group_id).unwrap();
    group.icon = Some("🚀".to_string());
    group.color = Some("#FF5733".to_string());
    test_db.db.update_group(&group).unwrap();

    let retrieved = test_db.db.get_group(group_id).unwrap();
    assert_eq!(retrieved.icon, Some("🚀".to_string()));
    assert_eq!(retrieved.color, Some("#FF5733".to_string()));
}


#[test]
fn test_get_groups_by_directory_basic() {
    let test_db = TestDb::setup_test_db();

    let mut group = GroupBuilder::new("Test").build();
    group.working_directory = Some(TestDb::get_temp_dir());
    let group_id = test_db.db.create_group(&group).unwrap();

    test_db.create_test_group("Other");

    let result = test_db.db.get_groups_by_directory(Some(&TestDb::get_temp_dir())).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, group_id);
    assert_eq!(result[0].working_directory, Some(TestDb::get_temp_dir()));
}

#[test]
fn test_get_groups_by_directory_none_returns_all_with_directory() {
    let test_db = TestDb::setup_test_db();

    let mut g1 = GroupBuilder::new("WithDir").build();
    g1.working_directory = Some(TestDb::get_temp_dir());
    test_db.db.create_group(&g1).unwrap();

    let mut g2 = GroupBuilder::new("WithDir2").build();
    g2.working_directory = Some("~".to_string());
    test_db.db.create_group(&g2).unwrap();

    let g3 = GroupBuilder::new("NoDir").build();
    test_db.db.create_group(&g3).unwrap();

    let result = test_db.db.get_groups_by_directory(None).unwrap();
    assert_eq!(result.len(), 1);
}

#[test]
fn test_get_groups_by_directory_empty_directory() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.get_groups_by_directory(Some("/nonexistent/path"));
    assert!(result.is_err());
}

#[test]
fn test_get_groups_by_directory_multiple_groups() {
    let test_db = TestDb::setup_test_db();

    let shared_dir = TestDb::get_temp_dir();

    let mut g1 = GroupBuilder::new("Group1").build();
    g1.working_directory = Some(shared_dir.clone());

    let mut g2 = GroupBuilder::new("Group2").build();
    g2.working_directory = Some(shared_dir.clone());

    let mut g3 = GroupBuilder::new("Group3").build();
    g3.working_directory = Some("~".to_string());

    let id1 = test_db.db.create_group(&g1).unwrap();
    let id2 = test_db.db.create_group(&g2).unwrap();
    test_db.db.create_group(&g3).unwrap();

    let result = test_db.db.get_groups_by_directory(Some(&shared_dir)).unwrap();
    assert_eq!(result.len(), 2);
    let ids: Vec<i64> = result.iter().map(|g| g.id).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}

#[test]
fn test_get_groups_by_directory_orders_by_position() {
    let test_db = TestDb::setup_test_db();
    let parent_id = test_db.create_test_group("Parent");

    let mut g1 = GroupBuilder::new("A").with_parent(parent_id).build();
    g1.working_directory = Some(TestDb::get_temp_dir());

    let mut g2 = GroupBuilder::new("B").with_parent(parent_id).build();
    g2.working_directory = Some(TestDb::get_temp_dir());

    test_db.db.create_group(&g1).unwrap();
    test_db.db.create_group(&g2).unwrap();

    let result = test_db.db.get_groups_by_directory(Some(&TestDb::get_temp_dir())).unwrap();
    assert_eq!(result.len(), 2);
    assert!(result[0].position < result[1].position);
}

#[test]
fn test_get_groups_by_directory_with_null_working_directory() {
    let test_db = TestDb::setup_test_db();

    let group = GroupBuilder::new("NoDir").build();
    test_db.db.create_group(&group).unwrap();

    let result = test_db.db.get_groups_by_directory(Some(""));
    assert!(result.is_err());
}

#[test]
fn test_replace_groups_directory_basic() {
    let test_db = TestDb::setup_test_db();

    let mut group = GroupBuilder::new("Test").build();
    group.working_directory = Some("~".to_string());
    let group_id = test_db.db.create_group(&group).unwrap();

    let affected = test_db.db.replace_groups_directory([group_id].to_vec(), Some(&TestDb::get_temp_dir())).unwrap();
    assert_eq!(affected, 1);

    let updated = test_db.db.get_group(group_id).unwrap();
    assert_eq!(updated.working_directory, Some(TestDb::get_temp_dir()));
}

#[test]
fn test_replace_groups_directory_to_none() {
    let test_db = TestDb::setup_test_db();

    let mut group = GroupBuilder::new("Test").build();
    group.working_directory = Some("~".to_string());
    let group_id = test_db.db.create_group(&group).unwrap();

    let affected = test_db.db.replace_groups_directory([group_id].to_vec(), None).unwrap();
    assert_eq!(affected, 1);

    let updated = test_db.db.get_group(group_id).unwrap();
    assert_eq!(updated.working_directory, None);
}

#[test]
fn test_replace_groups_directory_from_none() {
    let test_db = TestDb::setup_test_db();

    let group = GroupBuilder::new("Test").build();
    let group_id = test_db.db.create_group(&group).unwrap();

    let affected = test_db.db.replace_groups_directory([group_id].to_vec(), Some(&TestDb::get_temp_dir())).unwrap();
    assert_eq!(affected, 1);

    let updated = test_db.db.get_group(group_id).unwrap();
    assert_eq!(updated.working_directory, Some(TestDb::get_temp_dir()));
}

#[test]
fn test_replace_groups_directory_multiple_ids() {
    let test_db = TestDb::setup_test_db();

    let mut g1 = GroupBuilder::new("Group1").build();
    g1.working_directory = Some("~".to_string());

    let mut g2 = GroupBuilder::new("Group2").build();
    g2.working_directory = Some("~".to_string());

    let id1 = test_db.db.create_group(&g1).unwrap();
    let id2 = test_db.db.create_group(&g2).unwrap();

    let affected = test_db.db.replace_groups_directory([id1, id2].to_vec(), Some(&TestDb::get_temp_dir())).unwrap();
    assert_eq!(affected, 2);

    assert_eq!(test_db.db.get_group(id1).unwrap().working_directory, Some(TestDb::get_temp_dir()));
    assert_eq!(test_db.db.get_group(id2).unwrap().working_directory, Some(TestDb::get_temp_dir()));
}

#[test]
fn test_replace_groups_directory_empty_ids() {
    let test_db = TestDb::setup_test_db();
    let affected = test_db.db.replace_groups_directory([].to_vec(), Some("/new/dir")).unwrap();
    assert_eq!(affected, 0);
}

#[test]
fn test_replace_groups_directory_invalid_id_fails_all() {
    let test_db = TestDb::setup_test_db();

    let mut group = GroupBuilder::new("Test").build();
    group.working_directory = Some(TestDb::get_temp_dir());
    let valid_id = test_db.db.create_group(&group).unwrap();

    let result = test_db.db.replace_groups_directory([valid_id, 99999].to_vec(), Some(&TestDb::get_temp_dir()));
    assert!(result.is_err());

    // Verify valid_id was NOT updated (transaction rolled back)
    let unchanged = test_db.db.get_group(valid_id).unwrap();
    assert_eq!(unchanged.working_directory, Some(TestDb::get_temp_dir()));
}

#[test]
fn test_replace_groups_directory_updates_timestamp() {
    let test_db = TestDb::setup_test_db();

    let mut group = GroupBuilder::new("Test").build();
    group.working_directory = Some(TestDb::get_temp_dir());
    let group_id = test_db.db.create_group(&group).unwrap();

    let original = test_db.db.get_group(group_id).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1000));

    test_db.db.replace_groups_directory([group_id].to_vec(), Some("~")).unwrap();

    let updated = test_db.db.get_group(group_id).unwrap();
    assert_ne!(original.updated_at, updated.updated_at);
}

#[test]
fn test_replace_groups_directory_preserves_other_fields() {
    let test_db = TestDb::setup_test_db();
    let category_id = test_db.create_test_category("Test Category");

    let mut group = GroupBuilder::new("Full")
        .with_category(category_id)
        .with_env("KEY", "value")
        .build();
    group.description = Some("Description".to_string());
    group.working_directory = Some(TestDb::get_temp_dir());
    group.shell = Some("/bin/zsh".to_string());
    group.color = Some("#FF5733".to_string());
    let group_id = test_db.db.create_group(&group).unwrap();

    test_db.db.replace_groups_directory([group_id].to_vec(), Some("~")).unwrap();

    let updated = test_db.db.get_group(group_id).unwrap();
    assert_eq!(updated.name, group.name);
    assert_eq!(updated.category_id, group.category_id);
    assert_eq!(updated.env_vars, Some(HashMap::from([("KEY".to_string(), "value".to_string())])));
    assert_eq!(updated.description, group.description);
    assert_eq!(updated.shell, group.shell);
    assert_eq!(updated.icon, group.icon);
    assert_eq!(updated.color, group.color);
}

#[test]
fn test_replace_groups_directory_nonexistent_path_fails() {
    let test_db = TestDb::setup_test_db();

    let mut group = GroupBuilder::new("Test").build();
    group.working_directory = Some("~".to_string());
    let group_id = test_db.db.create_group(&group).unwrap();

    let result = test_db.db.replace_groups_directory([group_id].to_vec(), Some("/this/path/does/not/exist/anywhere"));
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData {
            field: "working_directory",
            ..
        })
    ));
}


#[test]
fn test_duplicate_groups_basic() {
    let test_db = TestDb::setup_test_db();

    let group = GroupBuilder::new("Original").build();
    let original_id = test_db.db.create_group(&group).unwrap();

    let new_ids = test_db.db.duplicate_groups([original_id].to_vec(), "Copy of ", false).unwrap();
    assert_eq!(new_ids.len(), 1);

    let original = test_db.db.get_group(original_id).unwrap();
    let duplicate = test_db.db.get_group(new_ids[0]).unwrap();

    assert_eq!(duplicate.name, "Copy of Original");
    assert_eq!(duplicate.parent_group_id, original.parent_group_id);
    assert_ne!(duplicate.id, original.id);
}

#[test]
fn test_duplicate_groups_empty_ids() {
    let test_db = TestDb::setup_test_db();
    let new_ids = test_db.db.duplicate_groups([].to_vec(), "Copy of ", false).unwrap();
    assert!(new_ids.is_empty());
}

#[test]
fn test_duplicate_groups_preserves_all_fields() {
    let test_db = TestDb::setup_test_db();
    let category_id = test_db.create_test_category("Test Category");

    let mut group = GroupBuilder::new("Full")
        .with_category(category_id)
        .with_env("KEY", "value")
        .build();
    group.description = Some("Description".to_string());
    group.working_directory = Some(TestDb::get_temp_dir());
    group.shell = Some("/bin/bash".to_string());
    group.icon = Some("🚀".to_string());
    group.color = Some("#FF5733".to_string());

    let original_id = test_db.db.create_group(&group).unwrap();
    let new_ids = test_db.db.duplicate_groups([original_id].to_vec(), "", false).unwrap();

    let duplicate = test_db.db.get_group(new_ids[0]).unwrap();
    assert_eq!(duplicate.description, group.description);
    assert_eq!(duplicate.working_directory, group.working_directory);
    assert_eq!(duplicate.shell, group.shell);
    assert_eq!(duplicate.category_id, group.category_id);
    assert_eq!(duplicate.env_vars, Some(HashMap::from([("KEY".to_string(), "value".to_string())])));
    assert_eq!(duplicate.icon, group.icon);
    assert_eq!(duplicate.color, group.color);
}

#[test]
fn test_duplicate_groups_resets_favorite() {
    let test_db = TestDb::setup_test_db();

    let mut group = GroupBuilder::new("Fav").build();
    group.is_favorite = true;
    let original_id = test_db.db.create_group(&group).unwrap();

    let new_ids = test_db.db.duplicate_groups([original_id].to_vec(), "Copy ", false).unwrap();

    let duplicate = test_db.db.get_group(new_ids[0]).unwrap();
    assert!(!duplicate.is_favorite);
}

#[test]
fn test_duplicate_groups_assigns_new_position() {
    let test_db = TestDb::setup_test_db();
    let parent_id = test_db.create_test_group("Parent");

    let group = GroupBuilder::new("Test").with_parent(parent_id).build();
    let original_id = test_db.db.create_group(&group).unwrap();

    let new_ids = test_db.db.duplicate_groups([original_id].to_vec(), "Copy ", false).unwrap();

    let original = test_db.db.get_group(original_id).unwrap();
    let duplicate = test_db.db.get_group(new_ids[0]).unwrap();

    assert_ne!(original.position, duplicate.position);
    assert!(duplicate.position > original.position);
}

#[test]
fn test_duplicate_groups_multiple() {
    let test_db = TestDb::setup_test_db();

    let g1 = GroupBuilder::new("Group1").build();
    let g2 = GroupBuilder::new("Group2").build();
    let g3 = GroupBuilder::new("Group3").build();

    let id1 = test_db.db.create_group(&g1).unwrap();
    let id2 = test_db.db.create_group(&g2).unwrap();
    let id3 = test_db.db.create_group(&g3).unwrap();

    let new_ids = test_db.db.duplicate_groups([id1, id2, id3].to_vec(), "Backup ", false).unwrap();
    assert_eq!(new_ids.len(), 3);

    assert_eq!(test_db.db.get_group(new_ids[0]).unwrap().name, "Backup Group1");
    assert_eq!(test_db.db.get_group(new_ids[1]).unwrap().name, "Backup Group2");
    assert_eq!(test_db.db.get_group(new_ids[2]).unwrap().name, "Backup Group3");
}

#[test]
fn test_duplicate_groups_invalid_id_fails_all() {
    let test_db = TestDb::setup_test_db();

    let group = GroupBuilder::new("Valid").build();
    let valid_id = test_db.db.create_group(&group).unwrap();

    let result = test_db.db.duplicate_groups([valid_id, 99999].to_vec(), "Copy ", false);
    assert!(result.is_err());

    // Verify no duplicate was created for valid_id
    let count = test_db.db.get_groups_count(None, None, false).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_duplicate_groups_empty_prefix() {
    let test_db = TestDb::setup_test_db();

    let group = GroupBuilder::new("Original").build();
    let original_id = test_db.db.create_group(&group).unwrap();

    let new_ids = test_db.db.duplicate_groups([original_id].to_vec(), "", false).unwrap();

    let duplicate = test_db.db.get_group(new_ids[0]).unwrap();
    assert_eq!(duplicate.name, "Original");
}

#[test]
fn test_duplicate_groups_with_special_chars_in_prefix() {
    let test_db = TestDb::setup_test_db();

    let group = GroupBuilder::new("Test").build();
    let original_id = test_db.db.create_group(&group).unwrap();

    let new_ids = test_db.db.duplicate_groups([original_id].to_vec(), "[BACKUP] ", false).unwrap();

    let duplicate = test_db.db.get_group(new_ids[0]).unwrap();
    assert_eq!(duplicate.name, "[BACKUP] Test");
}

#[test]
fn test_duplicate_groups_non_recursive() {
    let test_db = TestDb::setup_test_db();
    let parent_id = test_db.create_test_group("Parent");

    let child = GroupBuilder::new("Child").with_parent(parent_id).build();
    let child_id = test_db.db.create_group(&child).unwrap();

    let grandchild = GroupBuilder::new("Grandchild").with_parent(child_id).build();
    test_db.db.create_group(&grandchild).unwrap();

    // Non-recursive: only duplicates the specified group, not descendants
    let new_ids = test_db.db.duplicate_groups([child_id].to_vec(), "Copy ", false).unwrap();
    assert_eq!(new_ids.len(), 1);

    let duplicate = test_db.db.get_group(new_ids[0]).unwrap();
    assert_eq!(duplicate.name, "Copy Child");
    // Parent should be preserved (same parent as original)
    assert_eq!(duplicate.parent_group_id, Some(parent_id));
}

#[test]
fn test_duplicate_groups_recursive() {
    let test_db = TestDb::setup_test_db();
    let parent_id = test_db.create_test_group("Parent");

    let child_id = test_db.db.create_group(&GroupBuilder::new("Child").with_parent(parent_id).build()).unwrap();

    test_db.db.create_group(&GroupBuilder::new("Grandchild").with_parent(child_id).build()).unwrap();

    // Recursive: duplicates the group and all descendants
    let new_ids = test_db.db.duplicate_groups([child_id].to_vec(), "Copy ", true).unwrap();

    // Should have 2 new groups: Copy Child and Copy Grandchild
    assert_eq!(new_ids.len(), 2);

    // Find the new child and grandchild by name
    let new_child = test_db.db.get_group(new_ids[0]).unwrap();
    assert_eq!(new_child.name, "Copy Child");
    assert_eq!(new_child.parent_group_id, Some(parent_id));

    let new_grandchild = test_db.db.get_group(new_ids[1]).unwrap();
    assert_eq!(new_grandchild.name, "Grandchild"); // nested groups do not show copy
    // The grandchild should have the new child as parent, not the original
    assert_eq!(new_grandchild.parent_group_id, Some(new_ids[0]));
}

#[test]
fn test_duplicate_groups_recursive_multiple_roots() {
    let test_db = TestDb::setup_test_db();

    let root1_id = test_db.db.create_group(&GroupBuilder::new("Root1").build()).unwrap();
    test_db.db.create_group(&GroupBuilder::new("Child1").with_parent(root1_id).build()).unwrap();

    let root2_id = test_db.db.create_group(&GroupBuilder::new("Root2").build()).unwrap();
    test_db.db.create_group(&GroupBuilder::new("Child2").with_parent(root2_id).build()).unwrap();

    let new_ids = test_db.db.duplicate_groups([root1_id, root2_id].to_vec(), "Backup ", true).unwrap();
    assert_eq!(new_ids.len(), 4);

    let new_root1 = new_ids.iter().map(|id| test_db.db.get_group(*id).unwrap())
        .find(|g| g.name == "Backup Root1").unwrap();
    let new_child1 = new_ids.iter().map(|id| test_db.db.get_group(*id).unwrap())
        .find(|g| g.name == "Child1").unwrap();
    let new_root2 = new_ids.iter().map(|id| test_db.db.get_group(*id).unwrap())
        .find(|g| g.name == "Backup Root2").unwrap();
    let new_child2 = new_ids.iter().map(|id| test_db.db.get_group(*id).unwrap())
        .find(|g| g.name == "Child2").unwrap();

    assert_eq!(new_child1.parent_group_id, Some(new_root1.id));
    assert_eq!(new_child2.parent_group_id, Some(new_root2.id));
}

#[test]
fn test_duplicate_groups_recursive_preserves_hierarchy_depth() {
    let test_db = TestDb::setup_test_db();

    let level0 = GroupBuilder::new("Level0").build();
    let l0_id = test_db.db.create_group(&level0).unwrap();

    let level1 = GroupBuilder::new("Level1").with_parent(l0_id).build();
    let l1_id = test_db.db.create_group(&level1).unwrap();

    let level2 = GroupBuilder::new("Level2").with_parent(l1_id).build();
    let l2_id = test_db.db.create_group(&level2).unwrap();

    let level3 = GroupBuilder::new("Level3").with_parent(l2_id).build();
    test_db.db.create_group(&level3).unwrap();

    let new_ids = test_db.db.duplicate_groups([l0_id].to_vec(), "Copy ", true).unwrap();
    assert_eq!(new_ids.len(), 4);

    // Verify the chain: Copy Level3 -> Copy Level2 -> Copy Level1 -> Copy Level0
    let new_l3 = test_db.db.get_group(new_ids[3]).unwrap();
    let new_l2 = test_db.db.get_group(new_ids[2]).unwrap();
    let new_l1 = test_db.db.get_group(new_ids[1]).unwrap();
    let new_l0 = test_db.db.get_group(new_ids[0]).unwrap();

    assert_eq!(new_l3.parent_group_id, Some(new_ids[2]));
    assert_eq!(new_l2.parent_group_id, Some(new_ids[1]));
    assert_eq!(new_l1.parent_group_id, Some(new_ids[0]));
    assert_eq!(new_l0.parent_group_id, None);
}

#[test]
fn test_duplicate_groups_root_level() {
    let test_db = TestDb::setup_test_db();

    let group = GroupBuilder::new("RootGroup").build();
    let original_id = test_db.db.create_group(&group).unwrap();

    let new_ids = test_db.db.duplicate_groups([original_id].to_vec(), "Copy ", false).unwrap();

    let duplicate = test_db.db.get_group(new_ids[0]).unwrap();
    assert_eq!(duplicate.parent_group_id, None);
    assert_eq!(duplicate.name, "Copy RootGroup");
}

#[test]
fn test_duplicate_groups_in_group() {
    let test_db = TestDb::setup_test_db();
    let parent_id = test_db.create_test_group("Parent");

    let group = GroupBuilder::new("Child").with_parent(parent_id).build();
    let original_id = test_db.db.create_group(&group).unwrap();

    let new_ids = test_db.db.duplicate_groups([original_id].to_vec(), "Copy ", false).unwrap();

    let duplicate = test_db.db.get_group(new_ids[0]).unwrap();
    assert_eq!(duplicate.parent_group_id, Some(parent_id));
}

// ============================================================================
// Path Normalization Tests
// ============================================================================

#[test]
fn test_normalize_path_trailing_slash() {
    let test_db = TestDb::setup_test_db();

    let temp_dir = std::env::temp_dir();
    let temp_str = temp_dir.to_string_lossy().to_string();

    let mut group = GroupBuilder::new("Test").build();
    group.working_directory = Some(temp_str.clone());
    test_db.db.create_group(&group).unwrap();

    // Query with exact path should match
    let result = test_db.db.get_groups_by_directory(Some(&temp_str)).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_replace_groups_directory_validates_path_exists() {
    let test_db = TestDb::setup_test_db();

    let mut group = GroupBuilder::new("Test").build();
    group.working_directory = Some("~".to_string());
    let group_id = test_db.db.create_group(&group).unwrap();

    // Should fail for non-existent path
    let result = test_db.db.replace_groups_directory([group_id].to_vec(), Some("/definitely/not/real"));
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData {
            field: "working_directory",
            ..
        })
    ));
}
