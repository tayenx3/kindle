pub mod block;
pub mod entity;

use block::*;
use serde::{Serialize, Deserialize};
use entity::{Entity, RunBlockResult};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    name: String,
    value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    name: String,
    params: Vec<Variable>,
    body: Vec<Block>,
}

pub struct VMState {
    global_vars: Vec<Variable>,
    entities: Vec<Entity>
}

impl VMState {
    pub fn new() -> Self {
        Self {
            global_vars: Vec::new(),
            entities: Vec::new(),
        }
    }

    pub fn add_global_var(&mut self, var: Variable) {
        self.global_vars.push(var);
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn tick(&mut self, delta: f32) {
        let mut stop_all = false;
        for entity in &mut self.entities {
            let mut scripts = std::mem::take(&mut entity.scripts);
            
            for script in &mut scripts {
                if let RunBlockResult::StopAllScripts = script.tick(delta, &mut self.global_vars, entity) {
                    stop_all = true;
                }
            }
            
            entity.scripts = scripts;
        }

        if stop_all {
            for entity in &mut self.entities {
                entity.scripts.clear();
            }
        }
    }
}