use crate::framework::AllowedMethods;
use crate::internal::{request::Request, response::Response};

use crate::framework::radix::radix_trie::{RadixTrie, RouterError};

#[test]
fn test_get_prefix_between_two_strings() {
    let mut str_1 = "some";
    let mut str_2 = "something";

    let mut prefix = RadixTrie::get_prefix_between_two_strings(str_1, str_2).unwrap();

    assert_eq!(prefix, Some(String::from("some")));

    str_1 = "api/user";
    str_2 = "api/book";

    prefix = RadixTrie::get_prefix_between_two_strings(str_1, str_2).unwrap();
    assert_eq!(prefix, Some(String::from("api/")));

    str_1 = "api/user/id";
    str_2 = "api/user/x";

    prefix = RadixTrie::get_prefix_between_two_strings(str_1, str_2).unwrap();
    assert_eq!(prefix, Some(String::from("api/user/")));

    str_1 = "user/single";
    str_2 = "book/single";
    prefix = RadixTrie::get_prefix_between_two_strings(str_1, str_2).unwrap();
    assert_eq!(prefix, None);

    str_1 = "no_prefix";
    str_2 = "prefix_no";
    prefix = RadixTrie::get_prefix_between_two_strings(str_1, str_2).unwrap();
    assert_eq!(prefix, None)
}

fn trie_setup() -> RadixTrie {
    let mut trie = RadixTrie::new();
    // Add some default path
    fn handler(_req: &Request) -> Response {
        println!("Handler");
        Response::new(200, "Success", None)
    }

    trie.insert("/health", Some(AllowedMethods::GET), Some(handler));
    trie.insert("/api", None, None);
    trie.insert("/api/users", Some(AllowedMethods::POST), Some(handler));
    trie.insert("/api/users/", Some(AllowedMethods::GET), Some(handler));
    trie.insert("/api/users/:id", Some(AllowedMethods::GET), Some(handler));
    trie
}

#[test]
fn test_radix_trie_insert() {
    println!("TEST RADIX TRIE");

    let mut trie = RadixTrie::new();
    let root_api = "api";
    let health_route = "health";
    let get_users = "api/users";
    let get_active_users = "api/users/active";
    let get_books = "api/books";
    let book = "api/book";
    let upload = "api/uploads";
    let initiate_transaction = "api/transaction/initiate";
    let fetch_user_transaction = "api/transaction/user/:id";
    let fetch_user_single_transaction = "api/transaction/user/:id/single";
    let fetch_users_pending_transaction = "api/transaction/user/pending";

    fn handler(_req: &Request) -> Response {
        println!("This function takes a request and returns a response");
        Response::new(200, "Done", None)
    }

    trie.insert(root_api, None, None);
    trie.insert(health_route, Some(AllowedMethods::GET), Some(handler));
    trie.insert(get_users, Some(AllowedMethods::GET), Some(handler));
    trie.insert(get_books, Some(AllowedMethods::GET), Some(handler));
    trie.insert(get_active_users, Some(AllowedMethods::GET), Some(handler));
    trie.insert(book, Some(AllowedMethods::GET), Some(handler));
    trie.insert(book, Some(AllowedMethods::POST), Some(handler));
    trie.insert(upload, Some(AllowedMethods::POST), Some(handler));
    trie.insert(
        initiate_transaction,
        Some(AllowedMethods::POST),
        Some(handler),
    );
    trie.insert(
        fetch_user_transaction,
        Some(AllowedMethods::GET),
        Some(handler),
    );
    trie.insert(
        fetch_user_single_transaction,
        Some(AllowedMethods::GET),
        Some(handler),
    );
    trie.insert(
        fetch_users_pending_transaction,
        Some(AllowedMethods::GET),
        Some(handler),
    );

    let root_node = trie.get_root_node();
    let children_node = root_node.get_children();
    println!("Total child node => {}", children_node.len());

    // if let Some(child_api) = children_node.get(0) {
    //     let api_children = child_api.get_children();
    //     println!("all children path => {:?}", children_path);
    // }

    let children_path: Vec<String> = children_node[0]
        .child_nodes
        .iter()
        .map(|c| c.path.to_string())
        .collect();
    let input_child_paths = vec![
        String::from("users"),
        String::from("books"),
        String::from("transaction"),
        String::from("uploads"),
    ];

    println!(" childre path {:?}", children_path);

    // assert!(children_path.iter().any(|path| input_child_paths.contains(path)));
    assert_eq!(root_node.methods, None);

    assert!(children_node.len() >= 1);
    // assert!(children_node[0].path.starts_with("api"));

    let children_children = children_node[0].get_children();

    println!("\n\n");
    // root_node.print(1);
    assert!(children_children.len() > 1);
}

#[test]
fn test_radix_trie_search() {
    let trie = trie_setup();
    let search_key: &str = "/health";

    trie.get_root_node().print(3);
    let result = trie.search(search_key, AllowedMethods::GET);
    // println!("This is for no input => {:?} ", result);

    assert!(result.is_ok());
    let route_match = result.unwrap();
    assert!(route_match.params.is_empty());
    assert!(route_match.query.is_empty());
}

#[test]
fn test_radix_trie_search_with_param_input() {
    let trie = trie_setup();
    trie.get_root_node().print(3);

    let search_key = "/api/users/99";
    let result = trie.search(search_key, AllowedMethods::GET);

    assert!(result.is_ok());
    let route_match = result.unwrap();
    assert!(!route_match.params.is_empty());

    let route_param = route_match.params.get("id");

    assert_eq!(route_param, Some(&"99".to_string()));
}
//
#[test]
fn test_radix_trie_search_with_query_input() {
    let trie = trie_setup();
    let search_key = "/api/users?limit=60&active=true&sort=desc";
    let result = trie.search(search_key, AllowedMethods::GET);

    assert!(result.is_ok());
    let route_match = result.unwrap();
    let route_query = route_match.query;
    assert_eq!(route_query.get("limit"), Some(&"60".to_string()));
    assert_eq!(route_query.get("active"), Some(&"true".to_string()));
    assert_eq!(route_query.get("sort"), Some(&"desc".to_string()));
}
//
#[test]
fn test_radix_trie_search_with_param_and_query_input() {
    let trie = trie_setup();
    let search_key = "/api/users/99?limit=60&active=true&sort=desc";

    let result = trie.search(search_key, AllowedMethods::GET);
    assert!(result.is_ok());
    let route_match = result.unwrap();
    let route_param = route_match.params;
    let route_query = route_match.query;

    assert_eq!(route_query.get("active"), Some(&"true".to_string()));
    assert_eq!(route_query.get("sort"), Some(&"desc".to_string()));
    assert_eq!(route_param.get("id"), Some(&"99".to_string()));
}

#[test]
fn test_radix_trie_search_invalid_route() {
    let trie = trie_setup();
    let search_key = "/api/ghost/single";
    let result = trie.search(search_key, AllowedMethods::DELETE);

    assert!(result.is_err());
    assert!(matches!(result, Err(RouterError::RouteNotFound)));
}

#[test]
fn test_radix_trie_search_invalid_method() {
    let trie = trie_setup();
    let search_key = "/api/users";
    let result = trie.search(search_key, AllowedMethods::DELETE);

    assert!(result.is_err());
    assert!(matches!(result, Err(RouterError::MethodNotFound)));
}
