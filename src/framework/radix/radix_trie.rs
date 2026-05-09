use std::io::{Error, ErrorKind};

use crate::{framework::{
    radix::radix_node::{AllowedMethods, RadixNode},
    router::Handler,
}, internal::request};

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
    ) {
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
        let path = Self::normalize_path(request_path);

        // This gets updated per loop, so that the prefix check works
        let mut u_path: &str = path;

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
            u_path = Self::strip_slash_prefix(u_path);
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

                return;
            }

            let mut matched_any: bool = false;
            for (i, child) in node.child_nodes.iter().enumerate() {
                match Self::get_prefix_between_two_strings(u_path, &child.path) {
                    Ok(Some(_prefix)) => {
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
            }
            return;
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
            }

            node.add_child_node(new_node);
        };

        return;
    }

    pub fn search(&self, request_path: &str, method: Option<AllowedMethods>) -> Result<Handler, String> {}

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

    fn normalize_path(path: &str) -> &str {
        if path == "/" {
            return path;
        }

        path.trim_end_matches("/")
    }
}
