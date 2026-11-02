use daimon_harness::{Body, Decision, Event, Model, Priority, Session, ToolCall};
use daimon_minecraft::Sandbox;
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

const SNAPSHOT_LIMIT: u64 = 1_048_576;

struct Script(VecDeque<ToolCall>);
impl Model for Script {
    fn decide(&mut self, _: &Event, _: &Session) -> Result<Decision, String> {
        Ok(self.0.pop_front().map_or(Decision::Finish, Decision::Act))
    }
}

fn save(path: &Path, state: &Session) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = serde_json::to_vec_pretty(state)?;
    if bytes.len() as u64 > SNAPSHOT_LIMIT {
        return Err("snapshot capacity reached".into());
    }
    let temp = path.with_extension("tmp");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    std::fs::rename(&temp, path)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    File::open(parent)?.sync_all()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 || !["demo", "inspect"].contains(&args[1].as_str()) {
        return Err(
            "usage: daimoncraft demo NEW_DIRECTORY | inspect SNAPSHOT.json (offline only)".into(),
        );
    }
    if args[1] == "inspect" {
        let mut bytes = Vec::new();
        File::open(&args[2])?
            .take(SNAPSHOT_LIMIT + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > SNAPSHOT_LIMIT {
            return Err("snapshot too large".into());
        }
        let state: Session = serde_json::from_slice(&bytes)?;
        println!("{}", serde_json::to_string_pretty(&state)?);
        return Ok(());
    }
    // Exclusive new directory: rerunning can never overwrite or resume old effects.
    let directory = Path::new(&args[2]);
    std::fs::create_dir(directory)?;
    let snapshot = directory.join("session.json");
    let mut state = Session::new(12)?;
    let mut body = Sandbox::default();
    let calls: [(&str, Value); 8] = [
        ("observe", json!({})),
        ("step", json!({"dx":1,"dz":0})),
        ("observe", json!({})),
        (
            "chat",
            json!({"text":"Hello from the offline DaimonCraft sandbox."}),
        ),
        ("place", json!({"dx":0,"dz":1})),
        ("observe", json!({})),
        ("remove", json!({"dx":0,"dz":1})),
        ("observe", json!({})),
    ];
    let mut model = Script(
        calls
            .into_iter()
            .map(|(name, args)| ToolCall {
                name: format!("minecraft.{name}"),
                args,
            })
            .collect(),
    );
    let event = Event {
        source: "scripted-demo".into(),
        priority: Priority::Human,
        text: "Offline fixture; no real player, LLM or Matrix body.".into(),
    };
    loop {
        let action = state.prepare(&mut model, &event)?;
        save(&snapshot, &state)?;
        let Some(action) = action else { break };
        let evidence = body.execute(&action)?;
        println!("{}", serde_json::to_string(&evidence)?);
        state.reconcile(evidence)?;
        save(&snapshot, &state)?;
    }
    assert_eq!(body.position, (1, 0));
    assert!(body.blocks.is_empty());
    println!(
        "Offline sandbox complete: {} correlated outcomes; no live game connection.",
        state.evidence().len()
    );
    Ok(())
}
