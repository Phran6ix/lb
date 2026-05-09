use std::{
    collections::{HashMap, hash_map::Entry},
    // fmt::format,
};

use crate::framework::router::Handler;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AllowedMethods {
    GET,
    POST,
    PATCH,
    DELETE,
    PUT,
}

#[derive(Debug, Clone)]
pub struct RadixNode {
    pub path: String,
    pub child_nodes: Vec<RadixNode>,
    pub methods: Option<HashMap<AllowedMethods, Handler>>,
    pub param: bool,
}

impl RadixNode {
    pub fn new(path: String) -> Self {
        let param: bool = path.contains(':');

        RadixNode {
            path,
            child_nodes: vec![],
            methods: None,
            param,
        }
    }

    pub fn add_child_node(&mut self, child_node: RadixNode) -> &mut Self {
        self.child_nodes.push(child_node);
        self.child_nodes.last_mut().unwrap()
    }

    pub fn add_method(&mut self, method: AllowedMethods, handler: Handler) -> Result<(), String> {
        let method_map = self.methods.get_or_insert_with(HashMap::new);

        match method_map.entry(method) {
            Entry::Occupied(_) => Err(format!(
                "Method {:?} already exists on route {:?}",
                method, self.path
            )),
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(handler);
                return Ok(());
            }
        }
    }

    pub fn get_children(&self) -> &Vec<RadixNode> {
        &self.child_nodes
    }

    pub fn handover_children(&mut self, handover_to: &mut RadixNode) {
        // append the child nodes to the new nodes
        handover_to.child_nodes.append(&mut self.child_nodes);
        // clear the child nodes of the current node
        self.child_nodes.clear();
    }

    pub fn handover_method(&mut self, handover_to: &mut RadixNode) {
        // Rust will not allow an Option be empty at any time, even for a millisecond.
        // Which was the .take() method is for, it will transfer ownership and replace the
        // Option variable by None
        //
        // // handover_to.methods = self.methods;
        // // self.methods = None;
        handover_to.methods = self.methods.take();
    }

    pub fn update_node_path(&mut self, new_path: &str) {
        self.path = new_path.to_string();
    }

    pub fn print(&self, depth: usize) {
        let indent = "   ".repeat(depth);

        let star = if self.methods.is_some() { "*" } else { "" };
        let param = if self.param { "^" } else { "" };

        println!("{} └─ {}{}{}", indent, self.path, star, param);

        for child in &self.child_nodes {
            child.print(depth + 1);
        }
    }

    pub fn set_param(&mut self, param: bool) {
        self.param = param
    }
}
