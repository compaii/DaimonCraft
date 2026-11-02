//! A deterministic tiny voxel sandbox. This is NOT a Minecraft protocol client.
use daimon_harness::{Action, Body, Evidence};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Default)]
pub struct Sandbox {
    pub position: (i32, i32),
    pub blocks: BTreeSet<(i32, i32)>,
    pub chat: Vec<String>,
}

impl Sandbox {
    fn observe(&self) -> Value {
        json!({"kind":"sandbox", "position":[self.position.0,64,self.position.1], "blocks":self.blocks, "chat":self.chat})
    }
    fn offset(action: &Action) -> Result<(i32, i32), String> {
        let dx = action.call.args["dx"]
            .as_i64()
            .ok_or("dx must be an integer")?;
        let dz = action.call.args["dz"]
            .as_i64()
            .ok_or("dz must be an integer")?;
        if dx.abs_diff(0) > 1 || dz.abs_diff(0) > 1 || (dx == 0 && dz == 0) {
            return Err("offset must be one nonzero adjacent step".into());
        }
        Ok((dx as i32, dz as i32))
    }
    fn perform(&mut self, action: &Action) -> Result<(), String> {
        match action.call.name.as_str() {
            "minecraft.observe" => Ok(()),
            "minecraft.step" | "minecraft.place" | "minecraft.remove" => {
                let (dx, dz) = Self::offset(action)?;
                let target = (self.position.0 + dx, self.position.1 + dz);
                if target.0.abs() > 16 || target.1.abs() > 16 {
                    return Err("sandbox boundary".into());
                }
                match action.call.name.as_str() {
                    "minecraft.step" => {
                        if self.blocks.contains(&target) {
                            return Err("blocked".into());
                        }
                        self.position = target;
                    }
                    "minecraft.place" => {
                        if self.blocks.len() >= 32 {
                            return Err("block capacity reached".into());
                        }
                        if !self.blocks.insert(target) {
                            return Err("occupied".into());
                        }
                    }
                    _ => {
                        if !self.blocks.remove(&target) {
                            return Err("no block".into());
                        }
                    }
                }
                Ok(())
            }
            "minecraft.chat" => {
                let text = action.call.args["text"].as_str().ok_or("text required")?;
                if text.len() > 256 || self.chat.len() >= 8 {
                    return Err("chat capacity reached".into());
                }
                self.chat.push(text.into());
                Ok(())
            }
            _ => Err("unsupported tool".into()),
        }
    }
}

impl Body for Sandbox {
    fn execute(&mut self, action: &Action) -> Result<Evidence, String> {
        let result = self.perform(action);
        Ok(Evidence {
            action_id: action.id,
            success: result.is_ok(),
            observation: json!({"state":self.observe(), "error":result.err()}),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daimon_harness::ToolCall;
    fn action(name: &str, args: Value) -> Action {
        Action {
            id: 1,
            call: ToolCall {
                name: name.into(),
                args,
            },
        }
    }
    #[test]
    fn blocked_move_is_observed_and_reversible_change_restores_world() {
        let mut world = Sandbox::default();
        assert!(
            world
                .execute(&action("minecraft.place", json!({"dx":1,"dz":0})))
                .unwrap()
                .success
        );
        assert!(
            !world
                .execute(&action("minecraft.step", json!({"dx":1,"dz":0})))
                .unwrap()
                .success
        );
        assert_eq!(world.position, (0, 0));
        assert!(
            world
                .execute(&action("minecraft.remove", json!({"dx":1,"dz":0})))
                .unwrap()
                .success
        );
        assert!(world.blocks.is_empty());
        let result = world
            .execute(&action("minecraft.step", json!({"dx":1,"dz":0})))
            .unwrap();
        assert_eq!(result.observation["state"]["position"], json!([1, 64, 0]));
    }
    #[test]
    fn invalid_steps_do_not_mutate_position() {
        let mut world = Sandbox::default();
        for dx in [json!(2), json!(i64::MIN), json!(0.5), json!("1")] {
            assert!(
                !world
                    .execute(&action("minecraft.step", json!({"dx":dx,"dz":0})))
                    .unwrap()
                    .success
            );
            assert_eq!(world.position, (0, 0));
        }
    }
}
