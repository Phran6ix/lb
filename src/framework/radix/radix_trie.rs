use std::io::{Error, ErrorKind};

use crate::framework::{
    radix::radix_node::{AllowedMethods, RadixNode},
    router::Handler,
};

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
        println!("Original Path => {path}");
        // let mut prefixes: &str = "";

        let mut match_child_node_index: Option<usize> = None;

        // This gets updated per loop, so that the prefix check works
        let mut u_path: &str = path;
        // Used to slice the u_path
        let mut prefix_len: usize = 0;
        let mut partial_prefix: bool = false;
        let mut perfect_match: Option<usize> = None;
        // let mut bool: bool = false;

        // This is loop purpose is to get the node to add the child node. it breaks when it
        // encounters one of the three conditions

        let mut done: bool = false;
        

        'outer: loop {
            // DEAD END CONDITION
            println!("U_PATH => {u_path}");
            u_path = Self::strip_slash_prefix(u_path);
            if node.child_nodes.len() < 1 {
                println!("checking node => {}, inserting => {}", node.path, u_path);
                let node_path = Self::add_slash_suffix(u_path);
                let mut child_node = RadixNode::new(node_path);

                if method.is_some() && handler.is_some() {
                    if let (Some(m), Some(h)) = (method, handler)
                        && let Err(e) = child_node.add_method(m, h)
                    {
                        eprintln!("{:?}", e);
                        break;
                    }
                };
                node.add_child_node(child_node);
                let node_child = &node.get_children()[0];
                println!("Child => {}", node_child.path);

                return;
            }

            for (i, child) in node.child_nodes.iter().enumerate() {
                println!(
                    "checking the node path ( {} ) against the input {}",
                    child.path, u_path
                );
                // println!("{i}, {}", node.child_nodes.len());

                match Self::get_prefix_between_two_strings(u_path, &child.path) {
                    Ok(Some(_prefix)) => {
                        match_child_node_index = Some(i);
                        prefix_len = _prefix.len();

                        // Perfect Match
                        if prefix_len == child.path.len() && prefix_len == u_path.len() {
                            println!("This is a perfect fxking match");
                            perfect_match = Some(i);

                            done = true;
                            break;
                        }

                        // 1. complete prefix
                        if prefix_len == child.path.len() {
                            done = false;
                        // 2. partial prefix
                        } else {
                            partial_prefix = true;
                            done = true;
                        }

                        u_path = &u_path[prefix_len..];
                        node = &mut node.child_nodes[i];

                        break;
                    }
                    Ok(None) => {
                        println!("NO MORE PREFIX");
                        match_child_node_index = None;
                        prefix_len = 0;
                        done = true;
                        println!("No prefix, move to the next child");
                        continue;
                    }
                    Err(e) => {
                        done = true;
                        println!("Cannot perform prefix operation => {:?}", e);
                        break;
                    }
                };
            }

            if done {
                break 'outer;
            }
        }

        if let Some(perfect_match_index) = perfect_match {
            println!("Handle perfect match");
            let perfect_match_node = &mut node.child_nodes[perfect_match_index];
            if let Some((m, h)) = method.zip(handler)
                && let Err(e) = perfect_match_node.add_method(m, h)
            {
                eprintln!("{}", e);
                return;
            }
            return;
        }

        if partial_prefix {
            println!("PARTIAL PREFIX => {u_path}");
            let splitoff_path = node.path[prefix_len..].to_string();
            let way_path = node.path[..prefix_len].to_string();

            println!("splitoff => {}, waypath => {}", splitoff_path, way_path);

            let mut splitoff_node = RadixNode::new(Self::add_slash_suffix(&splitoff_path));

            if node.child_nodes.len() > 0 {
                node.handover_children(&mut splitoff_node);
            }

            if node.methods.is_some() {
                node.handover_method(&mut splitoff_node);
            }

            splitoff_node.set_param(node.param);

            node.update_node_path(&way_path);

            if u_path != "" {
                println!("new nodeee");
                let mut new_node = RadixNode::new(Self::add_slash_suffix(u_path));

                if let (Some(m), Some(h)) = (method, handler)
                    && let Err(e) = new_node.add_method(m, h)
                {
                    println!("{}", e);
                };
                node.add_child_node(new_node);
            }

            node.add_child_node(splitoff_node);
        } else {
            println!("Perfect match");
            println!(
                "We are dealing with this node {}, to insert {}",
                node.path, u_path
            );

            let mut new_node = RadixNode::new(Self::add_slash_suffix(u_path));
            if let Some((m, h)) = method.zip(handler)
                && let Err(e) = new_node.add_method(m, h)
            {
                println!("{}", e);
            }

            node.add_child_node(new_node);
        };

        println!(
            "prefix length {} \n child index {:?} \n path {} \n node path {}",
            prefix_len, match_child_node_index, path, node.path
        );
        println!("==================================================");
        return;
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

    pub fn normalize_path(path: &str) -> &str {
        if path == "/" {
            return path;
        }

        path.trim_end_matches("/")
    }
}
