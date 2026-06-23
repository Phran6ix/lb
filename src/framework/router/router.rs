use crate::{
    framework::radix::radix_trie::RadixTrie,
    internal::{
        request::{Request, RequestMethod},
        response::Response,
    },
};

pub type Handler = fn(&Request) -> Response;

pub struct Router {
    trie: RadixTrie,
}

impl Router {
    pub fn new() -> Self {
        Router {
            trie: RadixTrie::new(),
        }
    }

    pub fn get(&mut self, path: &str, handler: Handler) -> Result<(), String> {
        self.trie
            .insert(path, Some(RequestMethod::Get), Some(handler))
    }
    pub fn post(&mut self, path: &str, handler: Handler) -> Result<(), String> {
        self.trie
            .insert(path, Some(RequestMethod::Post), Some(handler))
    }
    pub fn patch(&mut self, path: &str, handler: Handler) -> Result<(), String> {
        self.trie
            .insert(path, Some(RequestMethod::Patch), Some(handler))
    }
    pub fn put(&mut self, path: &str, handler: Handler) -> Result<(), String> {
        self.trie
            .insert(path, Some(RequestMethod::Put), Some(handler))
    }
    pub fn delete(&mut self, path: &str, handler: Handler) -> Result<(), String> {
        self.trie
            .insert(path, Some(RequestMethod::Delete), Some(handler))
    }

    pub fn show_routes(&self) {
        self.trie.get_root_node().print(3);
    }

    pub fn resolve_path(
        &mut self,
        req: &mut Request,
        request_path: &str,
        method: &RequestMethod,
    ) -> Result<Handler, String> {
        let route = self
            .trie
            .search(request_path, method)
            .map_err(|e| e.to_string())?;

        req.set_param(route.params);
        req.set_query(route.query);

        Ok(*route.handler)
    }
}
