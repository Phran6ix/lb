use core::fmt;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
};

use crate::framework::{AllowedMethods, radix::radix_node::RadixNode, router::router::Handler};

#[derive(Debug)]
pub enum RouterError {
    MethodNotFound,
    RouteNotFound,
}

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterError::MethodNotFound => write!(f, "405: Method Not Allowed"),
            RouterError::RouteNotFound => write!(f, "404: Route Not Found"),
        }
    }
}

#[derive(Debug)]
pub struct RouteMatch<'a> {
    pub handler: &'a Handler,
    pub query: HashMap<String, String>,
    pub params: HashMap<String, String>,
}

pub struct RadixTrie {
    root_node: RadixNode,
}

impl RadixTrie {
    pub fn new() -> Self {
        RadixTrie {
            root_node: RadixNode::new(String::from("/")),
        }
    }

    pub fn get_root_node(&self) -> &RadixNode {
        &self.root_node
    }
    pub fn insert(
        &mut self,
        request_path: &str,
        method: Option<AllowedMethods>,
        handler: Option<Handler>,
    ) -> Result<(), String> {
        // To insert
        // Start from the root node
        // traverse down the tree based on the path  <----|
        // // get the child_nodes                         |
        // // check for prefix among the child nodes      |
        // ------------------------------------------------
        // loop until null or no prefix or end of path
        //
        // there are s conditions for a node to be inserted into the radix trie
        // 1. (DEAD END)it is at the end of the trie,  ie there are no child nodes
        // 2. (PERFECT MATCH)it has traverse the trie to a an existing node that is a path to a valid node
        // 3.  (FORK IN THE ROAD)partial match, eg /users and /uploads -> both have a prefix of (u) -> so it must
        //    split at this point

        let mut node: &mut RadixNode = &mut self.root_node;
        let path = &Self::normalize(request_path);

        // This gets updated per loop, so that the prefix check works
        let mut u_path: &str = &path;

        // Used to slice the u_path
        let mut prefix_len: usize = 0;
        let mut partial_prefix: bool = false;
        let mut perfect_match: Option<usize> = None;
        // let mut bool: bool = false;

        // This is loop purpose is to get the node to add the child node. it breaks when it
        // encounters one of the three conditions

        // let mut done: bool = false;

        'outer: loop {
            // DEAD END CONDITION

            if node.child_nodes.is_empty() {
                let node_path = Self::add_slash_suffix(u_path);
                let mut child_node = RadixNode::new(node_path);

                if let Some((m, h)) = method.zip(handler)
                    && let Err(e) = child_node.add_method(m, h)
                {
                    eprintln!("{:?}", e);
                    break;
                }
                node.add_child_node(child_node);

                return Ok(());
            }

            let mut matched_any: bool = false;
            for (i, child) in node.child_nodes.iter().enumerate() {
                match Self::get_prefix_between_two_strings(u_path, &child.path) {
                    Ok(Some(_prefix)) => {
                        if _prefix.starts_with(':') && child.path.starts_with(':') {
                            let u_param_name = u_path.split('/').next().unwrap();
                            let child_param_name = child.path.split('/').next().unwrap();

                            if u_param_name != child_param_name {
                                panic!(
                                    "Error: Conflicting parameter names at the same route level. \
             You already registered '{}', but you are trying to insert '{}'. \
             Full path: {}",
                                    child_param_name, u_param_name, &request_path
                                );
                            }
                        }
                        matched_any = true;
                        prefix_len = _prefix.len();

                        // Perfect Match
                        if prefix_len == child.path.len() && prefix_len == u_path.len() {
                            perfect_match = Some(i);
                            break;
                        } else if prefix_len == child.path.len() {
                            u_path = &u_path[prefix_len..];
                            node = &mut node.child_nodes[i];
                            break;
                        } else {
                            partial_prefix = true;
                            u_path = &u_path[prefix_len..];
                            node = &mut node.child_nodes[i];
                            break;
                        }
                    }
                    Ok(None) => {
                        prefix_len = 0;
                        // done = true;
                        continue;
                    }
                    Err(e) => {
                        eprintln!("{e}");
                        break;
                    }
                };
            }

            if !matched_any || partial_prefix || perfect_match.is_some() {
                break 'outer;
            }
        }

        if let Some(perfect_match_index) = perfect_match {
            let perfect_match_node = &mut node.child_nodes[perfect_match_index];
            if let Some((m, h)) = method.zip(handler)
                && let Err(e) = perfect_match_node.add_method(m, h)
            {
                eprintln!("{}", e);
                return Err(String::from(e));
            }
            return Ok(());
        }

        if partial_prefix {
            let splitoff_path = node.path[prefix_len..].to_string();
            let way_path = node.path[..prefix_len].to_string();

            let mut splitoff_node = RadixNode::new(Self::add_slash_suffix(&splitoff_path));

            if !node.child_nodes.is_empty() {
                node.handover_children(&mut splitoff_node);
            }

            if node.methods.is_some() {
                node.handover_method(&mut splitoff_node);
            }

            splitoff_node.set_param(node.param);

            node.update_node_path(&way_path);

            if !u_path.is_empty() {
                let mut new_node = RadixNode::new(Self::add_slash_suffix(u_path));

                if let Some((m, h)) = method.zip(handler)
                    && let Err(e) = new_node.add_method(m, h)
                {
                    println!("{}", e);
                };
                node.add_child_node(new_node);
            }

            node.add_child_node(splitoff_node);
        } else {
            let mut new_node = RadixNode::new(Self::add_slash_suffix(u_path));
            if let Some((m, h)) = method.zip(handler)
                && let Err(e) = new_node.add_method(m, h)
            {
                println!("{}", e);
                return Err(String::from(e));
            }

            node.add_child_node(new_node);
            return Ok(());
        };

        return Ok(());
    }

    pub fn search(
        &self,
        request_path: &str,
        method: AllowedMethods,
    ) -> Result<RouteMatch<'_>, RouterError> {
        // Step 1: Normalize the string, all url must end with a / since that is the normalizing
        // logic that the insert works with.

        let query_index = request_path.split_once('?');
        let (path_string, query_string) = match query_index {
            Some((p, q)) => (p, Some(q)),
            None => (request_path, None),
        };
        let normalized_path: String = Self::add_slash_suffix(path_string);
        let mut path: &str = &Self::strip_slash_prefix(&normalized_path);

        // step 2: Loop: Traverse the trie and look for diveregent - where the url and trie are prefix,
        // break the request path and traverse with the suffix
        // // Consider special patterns, eg dynamic param (xxx/:id) - determine the order of
        // pattern matching

        let mut params_vec: Vec<(&str, &str)> = Vec::new();
        let mut node: &RadixNode = &self.root_node;

        'outer: loop {
            let children = node.get_children();

            let mut param_node: Option<&RadixNode> = None;
            let mut matched_any: bool = false;

            for child in children.iter() {
                if child.path.starts_with(':') {
                    matched_any = false;
                    param_node = Some(child);
                    continue;
                }

                match path.strip_prefix(&child.path) {
                    Some(remaining_path) => {
                        node = child;
                        path = remaining_path;

                        matched_any = true;
                        break;
                    }
                    None => {
                        matched_any = false;
                    }
                };
            }

            if path == "/" && !matched_any {
                return Err(RouterError::RouteNotFound);
            }

            // if there was a match in the search, return the value
            if !matched_any {
                if let Some(p_node) = param_node {
                    let param_end_at = path.find("/").unwrap_or(path.len());
                    let p_node_path = &p_node.path[1..];

                    let param_key = match p_node_path.strip_suffix('/') {
                        Some(key) => key,
                        None => p_node_path,
                    };
                    let param_value = &path[..param_end_at];

                    params_vec.push((&param_key, param_value));

                    node = p_node;
                    path = &path[param_end_at..];
                } else {
                    break 'outer;
                }
            };

            if path.is_empty() || node.child_nodes.is_empty() || path == "/" {
                break 'outer;
            }
        }

        let Some(methods) = &node.methods else {
            return Err(RouterError::RouteNotFound);
        };

        let Some(handler) = methods.get(&method) else {
            return Err(RouterError::MethodNotFound);
        };

        let query: HashMap<String, String> = match query_string {
            None => HashMap::new(),
            Some(q) => q
                .split("&")
                .filter_map(|pair| pair.split_once("="))
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        };

        let params: HashMap<String, String> = params_vec
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Ok(RouteMatch {
            handler,
            query,
            params,
        })
    }

    pub fn get_prefix_between_two_strings(
        str_1: &str,
        str_2: &str,
    ) -> Result<Option<String>, Error> {
        let mut prefix: Vec<u8> = Vec::new();

        let str_1_bytes = str_1.as_bytes();
        let str_2_bytes = str_2.as_bytes();

        for (byte_1, byte_2) in str_1_bytes.iter().zip(str_2_bytes) {
            if byte_1 == byte_2 {
                prefix.push(*byte_1)
            } else {
                break;
            }
        }

        if prefix.len() == 0 {
            return Ok(None);
        } else if prefix.len() == 1 && prefix == b"/" {
            return Ok(None);
        } else {
            let prefix_string = match String::from_utf8(prefix) {
                Ok(s) => s,
                Err(e) => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Invalid UTF-8 sequence: {}", e),
                    ));
                }
            };
            return Ok(Some(prefix_string));
        }
    }

    fn add_slash_suffix(node_path: &str) -> String {
        if !node_path.ends_with("/") {
            format!("{}/", node_path)
        } else {
            node_path.to_string()
        }
    }
    fn strip_slash_prefix(node_path: &str) -> &str {
        let stripped = node_path.strip_prefix("/").unwrap_or(node_path);
        stripped
    }

    fn normalize(node_path: &str) -> String {
        let path: String;
        if !node_path.ends_with("/") {
            path = format!("{}/", node_path);
        } else {
            path = node_path.to_string();
        }

        let x = Self::strip_slash_prefix(&path);

        x.to_string()
    }
}
