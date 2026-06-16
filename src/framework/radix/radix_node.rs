use std::{
    collections::{HashMap, hash_map::Entry},
    // fmt::format,
};

use crate::framework::{AllowedMethods, router::router::Handler};

#[derive(Debug, Clone)]
pub struct RadixNode {
    pub path: String,
    pub child_nodes: Vec<RadixNode>,
    pub methods: Option<HashMap<AllowedMethods, Handler>>,
    pub param: bool,
}

impl RadixNode {
    pub fn new<S: Into<String>>(path: S) -> Self {
        let p: String = path.into();
        let param: bool = p.contains(':');

        RadixNode {
            path: p,
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
                "Method {:?} already exists on route {}",
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
        handover_to.child_nodes.reserve(self.child_nodes.len());
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

    pub fn insert_param_node(
        &mut self,
        path: &str,
        param_idx_start_at: usize,
    ) -> Result<( &mut Self, usize ), String> {
        if param_idx_start_at > 0 {
            let path_bytes = path.as_bytes();
            if path_bytes[param_idx_start_at - 1] != b'/' {
                return Err(format!(
                    "Syntax Error: Dynamic parameter ':' at position {} must be preceded by a forward slash '/' (e.g., '/:id')",
                    param_idx_start_at - 1
                ));
            }
        }

        let mut param_idx_end_at: usize = path[param_idx_start_at..]
            .find("/")
            .map(|idx| param_idx_start_at + idx)
            .unwrap_or(path.len());

        if param_idx_end_at < path.len() && path.as_bytes()[param_idx_end_at] == b'/' {
            param_idx_end_at += 1;
        }

        let param_path = &path[param_idx_start_at..param_idx_end_at];
        // println!("param path vs node path", );
        //
        // if param_path == self.path {
        //     println!("We are equal o");
        // }

        let path_before_param = &path[..param_idx_start_at];
        let mut param_node = RadixNode::new(param_path.to_string());

        param_node.set_param(true);
        let target_node = if !path_before_param.is_empty() {
            self.add_child_node(RadixNode::new(path_before_param.to_string()));
            self.child_nodes.last_mut().unwrap()
        } else {
            self
        };

        target_node.add_child_node(param_node);
        return Ok(( target_node.child_nodes.last_mut().unwrap(), param_idx_end_at ));
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

#[cfg(test)]
mod radix_node {
    use crate::internal::{request::Request, response::Response};

    use super::*;

    fn handler(_r: &Request) -> Response {
        Response::new(200, "OK", None)
    }

    fn setup_node() -> RadixNode {
        RadixNode::new("/")
    }

    #[test]
    fn test_create_radix_node_without_param() {
        let new_node = RadixNode::new("/new");
        assert_eq!(new_node.path, "/new");
        assert!(new_node.methods.is_none());
        assert!(!new_node.param);
    }

    #[test]
    fn test_create_radix_node_with_param() {
        let new_node = RadixNode::new("/:new");

        assert_eq!(new_node.path, "/:new");
        assert!(new_node.methods.is_none());

        assert!(new_node.param)
    }

    #[test]
    fn test_add_a_child_node_to_radix_node() {
        let mut node = setup_node();

        let child_node = RadixNode::new("/example");
        node.add_child_node(child_node);

        assert_eq!(node.child_nodes.len(), 1);
        let node_1 = node.child_nodes.get(0);
        assert!(node_1.is_some());

        if let Some(n) = node_1 {
            assert_eq!(n.path, "/example");
            assert!(n.methods.is_none());
            assert!(!n.param);
        }
    }

    #[test]
    fn test_add_multiple_child_nodes() {
        let mut node = setup_node();
        let child_node_1 = RadixNode::new("/example_one");
        let child_node_2 = RadixNode::new("/example_two");
        let child_node_3 = RadixNode::new("/example_three");

        node.add_child_node(child_node_1);
        node.add_child_node(child_node_2);
        node.add_child_node(child_node_3);

        assert_eq!(node.child_nodes.len(), 3);
    }

    #[test]
    fn test_add_child_node_with_method() {
        let mut node = setup_node();

        let child_node = RadixNode {
            path: "/example".to_string(),
            child_nodes: vec![],
            methods: Some(HashMap::from([(AllowedMethods::PATCH, handler as Handler)])),
            param: false,
        };

        node.add_child_node(child_node);

        assert_eq!(node.child_nodes.len(), 1);

        let Some(child) = node.child_nodes.get(0) else {
            panic!("TestFailed: No child node at index 0");
        };

        let Some(method) = &child.methods else {
            panic!("TestFailed: No Method is attached ")
        };

        assert!(
            method.contains_key(&AllowedMethods::PATCH),
            "Expected PATCH method to be registerd in node."
        );
    }

    #[test]
    fn test_add_method() {
        let mut node = RadixNode::new("/add_method");

        if let Err(e) = node.add_method(AllowedMethods::DELETE, handler) {
            panic!("TestFailed: Method was not inserted: {:?}", e)
        };

        let Some(method) = node.methods else {
            panic!("TestFailed: No method is registered on node");
        };

        assert!(method.get(&AllowedMethods::DELETE).is_some())
    }

    #[test]
    fn test_add_duplicate_method() {
        let mut node = RadixNode::new("/add_method");

        node.add_method(AllowedMethods::DELETE, handler).unwrap();

        let Err(actual_error) = node.add_method(AllowedMethods::DELETE, handler) else {
            panic!("Error expxected. ")
        };

        assert_eq!(
            actual_error,
            "Method DELETE already exists on route /add_method"
        )
    }

    #[test]
    fn test_handover_children() {
        let mut node = setup_node();
        let node_1 = RadixNode::new("/node_one");
        let node_2 = RadixNode::new("/node_two");
        let node_3 = RadixNode::new("/node_three");

        node.add_child_node(node_1);
        node.add_child_node(node_2);
        node.add_child_node(node_3);

        assert_eq!(node.child_nodes.len(), 3);

        let mut new_node = RadixNode::new("/new_node");
        assert_eq!(new_node.child_nodes.len(), 0);

        node.handover_children(&mut new_node);
        assert_eq!(node.child_nodes.len(), 0);
        assert_eq!(new_node.child_nodes.len(), 3)
    }

    #[test]
    fn test_handover_method() {
        let mut node = RadixNode::new("/handover");
        node.add_method(AllowedMethods::PUT, handler).unwrap();
        node.add_method(AllowedMethods::GET, handler).unwrap();
        node.add_method(AllowedMethods::POST, handler).unwrap();

        assert!(node.methods.is_some());
        let Some(methods) = &node.methods else {
            panic!("TestFailed: Expected a some value");
        };

        assert_eq!(methods.len(), 3);

        let mut new_node = RadixNode::new("/handover_to");
        assert!(new_node.methods.is_none());

        node.handover_method(&mut new_node);
        assert!(node.methods.is_none());
        let Some(new_methods) = &new_node.methods else {
            panic!("TestFailed: Expecting new methods");
        };

        assert_eq!(new_methods.len(), 3);
        assert!(new_methods.contains_key(&AllowedMethods::GET));
        assert!(new_methods.contains_key(&AllowedMethods::POST));
        assert!(new_methods.contains_key(&AllowedMethods::PUT));
    }

    #[test]
    fn test_update_node_path() {
        let mut node = RadixNode::new("/prev");

        assert_eq!(node.path, "/prev");

        node.update_node_path("/new_path");
        assert_eq!(node.path, "/new_path");
    }

    #[test]
    fn test_set_param() {
        let mut node = RadixNode::new("/prev");
        assert_eq!(node.param, false);

        node.set_param(true);
        assert_eq!(node.param, true);
    }
}
