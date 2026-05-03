use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Spawn the rustin binary with its cwd set to an isolated temp directory.
fn cmd(dir: &TempDir) -> Command {
    let mut c = Command::cargo_bin("rustin").unwrap();
    c.current_dir(dir.path());
    c
}

// ---------------------------------------------------------------------------
// init
// ---------------------------------------------------------------------------

#[test]
fn init_sets_board_title() {
    let dir = TempDir::new().unwrap();
    cmd(&dir)
        .args(["init", "MyProject"])
        .assert()
        .success()
        .stdout(predicate::str::contains("=== MyProject ==="));
}

#[test]
fn init_without_title_succeeds() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["init"]).assert().success();
}

#[test]
fn init_from_nested_directory_uses_project_root_for_title_and_file() {
    let dir = TempDir::new().unwrap();
    let project_dir = dir.path().join("project");
    let nested_dir = project_dir.join("subdir");
    fs::create_dir_all(&nested_dir).unwrap();
    fs::write(
        project_dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();

    let mut command = Command::cargo_bin("rustin").unwrap();
    command.current_dir(&nested_dir);
    command
        .args(["init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("=== project ==="));

    assert!(project_dir.join(".rustin.json").exists());
    assert!(!nested_dir.join(".rustin.json").exists());

    let board = fs::read_to_string(project_dir.join(".rustin.json")).unwrap();
    assert!(
        board.contains("\"title\": \"project\""),
        "board was:\n{board}"
    );
}

#[test]
fn init_in_git_repo_can_add_board_file_to_gitignore() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();

    cmd(&dir)
        .args(["init", "--gitignore", "add"])
        .assert()
        .success();

    let gitignore = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(gitignore.contains(".rustin.json"));
}

#[test]
fn init_in_git_repo_can_skip_gitignore_update() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();

    let mut command = Command::cargo_bin("rustin").unwrap();
    command.current_dir(dir.path());
    command.write_stdin("n\n");
    command.args(["init"]).assert().success();

    assert!(!dir.path().join(".gitignore").exists());
}

#[test]
fn init_in_git_repo_enter_defaults_to_skip_gitignore_update() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();

    let mut command = Command::cargo_bin("rustin").unwrap();
    command.current_dir(dir.path());
    command.write_stdin("\n");
    command.args(["init"]).assert().success();

    assert!(!dir.path().join(".gitignore").exists());
}

#[test]
fn init_in_git_repo_gitignore_skip_mode_does_not_update() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();

    cmd(&dir)
        .args(["init", "--gitignore", "skip"])
        .assert()
        .success();

    assert!(!dir.path().join(".gitignore").exists());
}

// ---------------------------------------------------------------------------
// list (empty board)
// ---------------------------------------------------------------------------

#[test]
fn list_empty_board_shows_message() {
    let dir = TempDir::new().unwrap();
    cmd(&dir)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("The board is empty"));
}

#[test]
fn list_with_malformed_board_shows_actionable_parse_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join(".rustin.json"), "{not-json").unwrap();

    cmd(&dir)
        .args(["list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to parse board file at"))
        .stderr(predicate::str::contains("rustin init"));
}

#[test]
fn no_args_shows_board() {
    let dir = TempDir::new().unwrap();
    cmd(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("The board is empty"));
}

// ---------------------------------------------------------------------------
// add
// ---------------------------------------------------------------------------

#[test]
fn add_creates_task_visible_in_todo_column() {
    let dir = TempDir::new().unwrap();
    cmd(&dir)
        .args(["add", "Write tests"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Write tests"))
        .stdout(predicate::str::contains("Todo"));
}

#[test]
fn add_with_high_priority_shows_fire_emoji() {
    let dir = TempDir::new().unwrap();
    cmd(&dir)
        .args(["add", "Critical fix", "-p", "high"])
        .assert()
        .success()
        .stdout(predicate::str::contains("🔥"));
}

#[test]
fn add_with_low_priority_shows_ice_emoji() {
    let dir = TempDir::new().unwrap();
    cmd(&dir)
        .args(["add", "Nice to have", "-p", "low"])
        .assert()
        .success()
        .stdout(predicate::str::contains("🧊"));
}

#[test]
fn add_with_bug_kind_shows_bug_emoji() {
    let dir = TempDir::new().unwrap();
    cmd(&dir)
        .args(["add", "Login crash", "-k", "bug"])
        .assert()
        .success()
        .stdout(predicate::str::contains("🐛"));
}

#[test]
fn add_with_chore_kind_shows_wrench_emoji() {
    let dir = TempDir::new().unwrap();
    cmd(&dir)
        .args(["add", "Update deps", "-k", "chore"])
        .assert()
        .success()
        .stdout(predicate::str::contains("🔧"));
}

#[test]
fn add_default_kind_is_feature_with_sparkle_emoji() {
    let dir = TempDir::new().unwrap();
    cmd(&dir)
        .args(["add", "New feature"])
        .assert()
        .success()
        .stdout(predicate::str::contains("✨"));
}

#[test]
fn add_multiple_tasks_assigns_incrementing_ids() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "First"]).assert().success();
    cmd(&dir).args(["add", "Second"]).assert().success();
    // Both should appear
    let out = cmd(&dir)
        .args(["list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("[ 1]"), "expected id 1, got:\n{s}");
    assert!(s.contains("[ 2]"), "expected id 2, got:\n{s}");
}

// ---------------------------------------------------------------------------
// remove
// ---------------------------------------------------------------------------

#[test]
fn remove_deletes_task_from_list() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Task to remove"]).assert().success();
    cmd(&dir)
        .args(["remove", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task to remove").not());
}

#[test]
fn remove_nonexistent_task_still_exits_ok() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["remove", "999"]).assert().success();
}

// ---------------------------------------------------------------------------
// inprogress / todo / done
// ---------------------------------------------------------------------------

#[test]
fn inprogress_moves_task_out_of_todo() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "In-flight task"]).assert().success();
    cmd(&dir).args(["inprogress", "1"]).assert().success();

    // Must NOT appear in todo-only view
    cmd(&dir)
        .args(["list", "-c", "todo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("In-flight task").not());

    // MUST appear in inprogress-only view
    cmd(&dir)
        .args(["list", "-c", "inprogress"])
        .assert()
        .success()
        .stdout(predicate::str::contains("In-flight task"));
}

#[test]
fn todo_moves_task_back_to_todo() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Back to todo"]).assert().success();
    cmd(&dir).args(["inprogress", "1"]).assert().success();
    cmd(&dir).args(["todo", "1"]).assert().success();

    cmd(&dir)
        .args(["list", "-c", "todo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Back to todo"));
}

#[test]
fn done_moves_task_to_done_column() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Finished task"]).assert().success();
    cmd(&dir).args(["done", "1"]).assert().success();

    cmd(&dir)
        .args(["list", "-c", "done"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Finished task"));
}

#[test]
fn done_task_not_in_todo_column() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Done task"]).assert().success();
    cmd(&dir).args(["done", "1"]).assert().success();

    cmd(&dir)
        .args(["list", "-c", "todo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Done task").not());
}

#[test]
fn inprogress_nonexistent_task_keeps_existing_tasks_unchanged() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Still todo"]).assert().success();

    cmd(&dir).args(["inprogress", "999"]).assert().success();

    cmd(&dir)
        .args(["list", "-c", "todo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Still todo"));
}

#[test]
fn todo_nonexistent_task_keeps_existing_tasks_unchanged() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Still done"]).assert().success();
    cmd(&dir).args(["done", "1"]).assert().success();

    cmd(&dir).args(["todo", "999"]).assert().success();

    cmd(&dir)
        .args(["list", "-c", "done"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Still done"));
}

#[test]
fn done_nonexistent_task_keeps_existing_tasks_unchanged() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Still todo"]).assert().success();

    cmd(&dir).args(["done", "999"]).assert().success();

    cmd(&dir)
        .args(["list", "-c", "todo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Still todo"));
}

// ---------------------------------------------------------------------------
// list --columns / -c filter
// ---------------------------------------------------------------------------

#[test]
fn list_all_columns_header_present() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Any task"]).assert().success();
    cmd(&dir)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Todo"))
        .stdout(predicate::str::contains("In Progress"))
        .stdout(predicate::str::contains("Done"));
}

#[test]
fn list_columns_todo_hides_other_columns() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Task"]).assert().success();
    cmd(&dir)
        .args(["list", "-c", "todo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Todo"))
        .stdout(predicate::str::contains("In Progress").not())
        .stdout(predicate::str::contains("Done").not());
}

#[test]
fn list_columns_multiple_shows_selected_only() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Task 1"]).assert().success();
    cmd(&dir).args(["done", "1"]).assert().success();
    cmd(&dir).args(["add", "Task 2"]).assert().success();

    cmd(&dir)
        .args(["list", "-c", "todo", "done"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Todo"))
        .stdout(predicate::str::contains("Done"))
        .stdout(predicate::str::contains("In Progress").not());
}

#[test]
fn list_from_nested_directory_uses_parent_board_file() {
    let dir = TempDir::new().unwrap();
    let nested = dir.path().join("nested").join("deep");
    fs::create_dir_all(&nested).unwrap();
    fs::write(
        dir.path().join(".rustin.json"),
        r#"{
    "version": "0.0.0",
    "title": "ParentBoard",
    "next_id": 2,
    "tasks": [
        {
            "id": 1,
            "title": "From parent",
            "priority": "medium",
            "kind": "feature",
            "description": null,
            "status": "todo",
            "created_at": "2024-01-01T00:00:00Z",
            "transitions": []
        }
    ]
}"#,
    )
    .unwrap();

    let mut command = Command::cargo_bin("rustin").unwrap();
    command.current_dir(&nested);
    command
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("=== ParentBoard ==="))
        .stdout(predicate::str::contains("From parent"));
}

#[test]
fn list_sorts_equal_priority_and_timestamp_by_id() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join(".rustin.json"),
        r#"{
    "version": "0.0.0",
    "title": "SortBoard",
    "next_id": 3,
    "tasks": [
        {
            "id": 2,
            "title": "Second",
            "priority": "medium",
            "kind": "feature",
            "description": null,
            "status": "todo",
            "created_at": "2024-01-01T00:00:00Z",
            "transitions": []
        },
        {
            "id": 1,
            "title": "First",
            "priority": "medium",
            "kind": "feature",
            "description": null,
            "status": "todo",
            "created_at": "2024-01-01T00:00:00Z",
            "transitions": []
        }
    ]
}"#,
    )
    .unwrap();

    let output = cmd(&dir)
        .args(["list", "-c", "todo"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).unwrap();
    let first = text.find("[ 1]").expect("expected first task in output");
    let second = text.find("[ 2]").expect("expected second task in output");
    assert!(first < second, "expected id 1 before id 2, got:\n{text}");
}

// ---------------------------------------------------------------------------
// show
// ---------------------------------------------------------------------------

#[test]
fn show_displays_all_fields() {
    let dir = TempDir::new().unwrap();
    cmd(&dir)
        .args(["add", "Build feature", "-p", "high", "-k", "chore"])
        .assert()
        .success();
    cmd(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ID:          1"))
        .stdout(predicate::str::contains("Title:       Build feature"))
        .stdout(predicate::str::contains("Kind:        Chore 🔧"))
        .stdout(predicate::str::contains("Priority:    High 🔥"))
        .stdout(predicate::str::contains("Status:      Todo"));
}

#[test]
fn show_displays_description_when_set() {
    let dir = TempDir::new().unwrap();
    cmd(&dir)
        .args(["add", "Task with desc", "-d", "Detailed note"])
        .assert()
        .success();
    cmd(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Description: Detailed note"));
}

#[test]
fn show_displays_status_history_after_transitions() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Task"]).assert().success();
    cmd(&dir).args(["inprogress", "1"]).assert().success();
    cmd(&dir).args(["done", "1"]).assert().success();
    cmd(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("History:"))
        .stdout(predicate::str::contains("Todo → In Progress"))
        .stdout(predicate::str::contains("In Progress → Done"));
}

#[test]
fn show_status_label_inprogress_is_human_readable() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Active task"]).assert().success();
    cmd(&dir).args(["inprogress", "1"]).assert().success();
    cmd(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status:      In Progress"));
}

// ---------------------------------------------------------------------------
// stat
// ---------------------------------------------------------------------------

#[test]
fn stat_summarizes_completed_inprogress_time() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join(".rustin.json"),
        r#"{
    "version": "0.0.0",
    "title": "StatsBoard",
    "next_id": 3,
    "tasks": [
        {
            "id": 1,
            "title": "Write release notes",
            "priority": "medium",
            "kind": "feature",
            "description": null,
            "status": "done",
            "created_at": "2024-01-01T09:00:00Z",
            "transitions": [
                { "from": "todo", "to": "in-progress", "at": "2024-01-01T10:00:00Z" },
                { "from": "in-progress", "to": "done", "at": "2024-01-01T11:30:00Z" }
            ]
        },
        {
            "id": 2,
            "title": "Clean up warnings",
            "priority": "medium",
            "kind": "feature",
            "description": null,
            "status": "done",
            "created_at": "2024-01-01T09:00:00Z",
            "transitions": [
                { "from": "todo", "to": "in-progress", "at": "2024-01-01T12:00:00Z" },
                { "from": "in-progress", "to": "done", "at": "2024-01-01T12:45:00Z" }
            ]
        }
    ]
}"#,
    )
    .unwrap();

    cmd(&dir)
        .args(["stat"])
        .assert()
        .success()
        .stdout(predicate::str::contains("=== StatsBoard stats ==="))
        .stdout(predicate::str::contains("Tasks:           2"))
        .stdout(predicate::str::contains("Completed runs:  2"))
        .stdout(predicate::str::contains("Total active:    2h 15m 00s"))
        .stdout(predicate::str::contains("[1] Write release notes"))
        .stdout(predicate::str::contains("1h 30m 00s"))
        .stdout(predicate::str::contains("[2] Clean up warnings"))
        .stdout(predicate::str::contains("45m 00s"))
        .stdout(predicate::str::contains("runs:1"))
        .stdout(predicate::str::contains("|████████████████|"))
        .stdout(predicate::str::contains("|████████░░░░░░░░|"));
}

#[test]
fn alias_st_runs_stat_command() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Task"]).assert().success();

    cmd(&dir)
        .args(["st"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Per task:"));
}

#[test]
fn show_nonexistent_task_exits_nonzero() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["show", "999"]).assert().failure();
}

// ---------------------------------------------------------------------------
// edit
// ---------------------------------------------------------------------------

#[test]
fn edit_updates_title() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Old title"]).assert().success();
    cmd(&dir)
        .args(["edit", "1", "--title", "New title"])
        .assert()
        .success();
    cmd(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Title:       New title"));
}

#[test]
fn edit_updates_priority() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Task"]).assert().success();
    cmd(&dir)
        .args(["edit", "1", "-p", "high"])
        .assert()
        .success();
    cmd(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Priority:    High 🔥"));
}

#[test]
fn edit_updates_kind() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Task"]).assert().success();
    cmd(&dir)
        .args(["edit", "1", "-k", "bug"])
        .assert()
        .success();
    cmd(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Kind:        Bug 🐛"));
}

#[test]
fn edit_sets_description() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["add", "Task"]).assert().success();
    cmd(&dir)
        .args(["edit", "1", "-d", "Added later"])
        .assert()
        .success();
    cmd(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Description: Added later"));
}

#[test]
fn edit_clears_description_with_empty_string() {
    let dir = TempDir::new().unwrap();
    cmd(&dir)
        .args(["add", "Task", "-d", "Initial desc"])
        .assert()
        .success();
    cmd(&dir).args(["edit", "1", "-d", ""]).assert().success();
    cmd(&dir)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Description:").not());
}

#[test]
fn edit_nonexistent_task_exits_nonzero() {
    let dir = TempDir::new().unwrap();
    cmd(&dir)
        .args(["edit", "999", "--title", "Ghost"])
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// Command aliases
// ---------------------------------------------------------------------------

#[test]
fn alias_a_adds_task() {
    let dir = TempDir::new().unwrap();
    cmd(&dir)
        .args(["a", "Via alias"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Via alias"));
}

#[test]
fn alias_l_lists_board() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["l"]).assert().success();
}

#[test]
fn alias_r_removes_task() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["a", "Temp"]).assert().success();
    cmd(&dir)
        .args(["r", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Temp").not());
}

#[test]
fn alias_ip_moves_to_inprogress() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["a", "Task"]).assert().success();
    cmd(&dir).args(["ip", "1"]).assert().success();
    cmd(&dir)
        .args(["l", "-c", "inprogress"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task"));
}

#[test]
fn alias_t_moves_to_todo() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["a", "Task"]).assert().success();
    cmd(&dir).args(["ip", "1"]).assert().success();
    cmd(&dir).args(["t", "1"]).assert().success();
    cmd(&dir)
        .args(["l", "-c", "todo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task"));
}

#[test]
fn alias_d_marks_done() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["a", "Task"]).assert().success();
    cmd(&dir).args(["d", "1"]).assert().success();
    cmd(&dir)
        .args(["l", "-c", "done"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task"));
}

#[test]
fn alias_s_shows_task() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["a", "My task"]).assert().success();
    cmd(&dir)
        .args(["s", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("My task"));
}

#[test]
fn alias_e_edits_task() {
    let dir = TempDir::new().unwrap();
    cmd(&dir).args(["a", "Old"]).assert().success();
    cmd(&dir)
        .args(["e", "1", "--title", "Updated"])
        .assert()
        .success();
    cmd(&dir)
        .args(["s", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated"));
}
