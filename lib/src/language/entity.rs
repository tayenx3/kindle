use super::block::*;
use super::{Variable, Function};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position { pub x: f32, pub y: f32 }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub pos: Position,
    pub rot: f32,
    pub size: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RunBlockResult {
    Normal,
    Wait(f32),
    StopThisScript,
    StopOtherScripts,
    StopAllScriptsInThisEntity,
    StopAllScripts,
    EnterScope(Vec<Block>)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityData {
    pub transform: Transform,
    pub vars: Vec<Variable>,
    pub hidden: bool,
}

enum ScriptState {
    Running,
    Waiting(f32),
    Stopped,
}

impl Script {
    pub fn tick(&mut self, delta: f32, globals: &mut [Variable], entity: &mut Entity) -> RunBlockResult {
        match &mut self.state {
            ScriptState::Running => {
                let (mut frame_idx, mut block_idx) = (0, 0);
                for (idx, frame) in self.frames.iter_mut().enumerate() {
                    frame_idx = idx;
                    block_idx = frame.index;
                }
                    self.frames[frame_idx].index += 1;
                while let Some(frame) = self.frames.last() {
                    if frame.index >= frame.blocks.len() {
                        self.frames.pop();
                    } else {
                        break;
                    }
                }

                let result = entity.run_block(&self.frames[frame_idx].blocks[block_idx], &[], globals);
                
                match result {
                    RunBlockResult::Normal => self.frames[frame_idx].index += 1,
                    RunBlockResult::Wait(secs) => self.state = ScriptState::Waiting(secs),
                    RunBlockResult::StopThisScript => self.state = ScriptState::Stopped,
                    RunBlockResult::StopAllScriptsInThisEntity => entity.scripts.clear(),
                    RunBlockResult::StopOtherScripts => {
                        self.other_flag = true;

                        entity.scripts.retain(|s| s.other_flag); // if others have the flag, they would've killed this script already

                        self.other_flag = false; // be the sigma (the only one left after the massacre)
                    },
                    RunBlockResult::StopAllScripts => return result,
                    RunBlockResult::EnterScope(blocks) => self.frames.push(Frame { blocks, index: 0 }),
                }
            },
            ScriptState::Waiting(remaining) => {
                *remaining -= delta;
                if *remaining <= 0.0 {
                    self.state = ScriptState::Running;
                }
            },
            ScriptState::Stopped => {}
        }

        RunBlockResult::Normal
    }
}

struct Frame {
    blocks: Vec<Block>,
    index: usize,
}

pub struct Script {
    frames: Vec<Frame>,
    state: ScriptState,
    other_flag: bool,
}

pub struct Entity {
    pub data: EntityData,
    pub functions: Vec<Function>,
    pub scripts: Vec<Script>,
}

impl Entity {
    pub fn run_block(&mut self, block: &Block, _functions: &[Function], globals: &mut [Variable]) -> RunBlockResult {
        match block {
            Block::ChangeXBy(d) => {
                let val = self.eval_expr(d, globals);
                self.data.transform.pos.x += val.as_f32();
            },
            Block::ChangeYBy(d) => {
                let val = self.eval_expr(d, globals);
                self.data.transform.pos.y += val.as_f32();
            },
            Block::SetPositionTo { x, y } => {
                let x = self.eval_expr(x, globals);
                let y = self.eval_expr(y, globals);
                self.data.transform.pos.x = x.as_f32();
                self.data.transform.pos.y = y.as_f32();
            },
            Block::ChangeVarBy { name, value } => {
                let val = self.eval_expr(value, globals);
                self.data.transform.pos.x += val.as_f32();

                self.data.vars.iter_mut()
                    .find(|var| var.name == *name)
                    .unwrap_or(
                        globals.iter_mut()
                            .find(|var| var.name == *name)
                            .unwrap()
                    )
                    .value.change_by(&val);
            },
            Block::SetVarTo { name, value } => {
                let val = self.eval_expr(value, globals);
                self.data.transform.pos.x += val.as_f32();

                self.data.vars.iter_mut()
                    .find(|var| var.name == *name)
                    .unwrap_or(
                        globals.iter_mut()
                            .find(|var| var.name == *name)
                            .unwrap()
                    )
                    .value = val;
            },
            Block::Show => self.data.hidden = false,
            Block::Hide => self.data.hidden = true,
            Block::WaitSeconds(expr) => {
                let result = self.eval_expr(expr, globals);
                return RunBlockResult::Wait(result.as_f32());
            },
            Block::If { condition, then } => {
                let result = self.eval_expr(condition, globals).as_bool();

                if result {
                    return RunBlockResult::EnterScope(then.clone());
                }
            },
            Block::IfElse { condition, then, else_ } => {
                let result = self.eval_expr(condition, globals).as_bool();

                if result {
                    return RunBlockResult::EnterScope(then.clone());
                } else {
                    return RunBlockResult::EnterScope(else_.clone());
                }
            },
            Block::StopThisScript => return RunBlockResult::StopThisScript,
            Block::StopAllScriptsInThisEntity => return RunBlockResult::StopAllScriptsInThisEntity,
            Block::StopOtherScripts => return RunBlockResult::StopOtherScripts,
            Block::StopAllScripts => return RunBlockResult::StopAllScripts,
        }
        RunBlockResult::Normal
    }

    pub fn eval_expr(&mut self, expr: &Expression, globals: &mut [Variable]) -> Value {
        match expr {
            Expression::Value(n) => n.clone(),
            Expression::Variable(n) => self.data.vars.iter()
                .find(|var| var.name == *n)
                .unwrap()
                .value.clone(),
            Expression::GlobalVar(n) => globals.iter()
                .find(|var| var.name == *n)
                .unwrap()
                .value.clone(),
        }
    }
}