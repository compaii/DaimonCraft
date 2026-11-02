use std::{fs, process::Command};

#[test]
fn offline_journey_persists_results_and_refuses_overwrite() {
    let parent = std::env::temp_dir().join(format!("daimoncraft-journey-{}", std::process::id()));
    fs::create_dir(&parent).unwrap();
    let run = parent.join("run");
    let binary = env!("CARGO_BIN_EXE_daimoncraft");
    let first = Command::new(binary).arg("demo").arg(&run).output().unwrap();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let snapshot = run.join("session.json");
    let before = fs::read(&snapshot).unwrap();
    let state: serde_json::Value = serde_json::from_slice(&before).unwrap();
    assert_eq!(state["evidence"].as_array().unwrap().len(), 8);
    assert_eq!(state["pending"], serde_json::Value::Null);
    assert_eq!(state["finished"], true);
    let last = &state["evidence"][7]["observation"]["state"];
    assert_eq!(last["position"], serde_json::json!([1, 64, 0]));
    assert!(last["blocks"].as_array().unwrap().is_empty());
    assert!(
        !Command::new(binary)
            .arg("demo")
            .arg(&run)
            .output()
            .unwrap()
            .status
            .success()
    );
    let read = Command::new(binary)
        .arg("inspect")
        .arg(&snapshot)
        .output()
        .unwrap();
    assert!(read.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&read.stdout).unwrap(),
        state
    );
    assert_eq!(fs::read(&snapshot).unwrap(), before);
    fs::remove_dir_all(parent).unwrap();
}
