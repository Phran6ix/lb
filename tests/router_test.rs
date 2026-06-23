use lb::Request;
use lb::Response;
use lb::Router;
use lb::framework::radix::radix_trie::RouterError;
use lb::internal::request::RequestMethod;

fn mock_response(_req: &Request) -> Response {
    Response::new(200, "OK", None)
}

#[test]
fn test_router_resolves_static_path() {
    let mut router = Router::new();

    router.get("/api/v1/dashboard", mock_response).unwrap();

    let mut mock_req = Request::new();

    let result = router.resolve_path(&mut mock_req, "/api/v1/dashboard", &RequestMethod::Get);

    assert!(
        result.is_ok(),
        "Expected an Ok result but got {:?}",
        result.err()
    );

    assert!(mock_req.param.is_none());
    assert!(mock_req.query.is_none());

    let resolved_handler = result.unwrap();
    let res = resolved_handler(&mock_req);
    assert_eq!(res.status_code, 200);
}

#[test]
fn test_router_resolves_dynamic_path() {
    let mut router = Router::new();
    router.post("/api/v1/user/:id", mock_response).unwrap();
    let mut mock_req = Request::new();

    // router.show_routes();
    let result = router.resolve_path(&mut mock_req, "/api/v1/user/usserid/", &RequestMethod::Post);

    assert!(result.is_ok());
    assert!(mock_req.param.is_some());
    assert!(mock_req.query.is_none());

    let Some(param) = mock_req.param else {
        panic!("TestFailed: Expected some param value")
    };

    let user_param = param.get("id");
    assert_eq!(user_param, Some(&"usserid".to_string()));
}

#[test]
fn test_router_resolves_static_and_dynamic_path() {
    let mut router = Router::new();

    //static rouers
    router.post("/api/user", mock_response).unwrap();
    router.get("/api/users", mock_response).unwrap();

    // dynamic routers
    router
        .post("/api/user/:userId/post", mock_response)
        .unwrap();
    router
        .get("/api/user/:userId/posts", mock_response)
        .unwrap();
    router
        .get("/api/user/:userId/post/:postId", mock_response)
        .unwrap();

    // router.show_routes();
    let mut request = Request::new();

    let mut result = router.resolve_path(&mut request, "/api/user", &RequestMethod::Post);
    assert!(result.is_ok());
    result = router.resolve_path(&mut request, "/api/users", &RequestMethod::Get);
    assert!(result.is_ok());
    result = router.resolve_path(&mut request, "/api/user/userid/post", &RequestMethod::Post);
    assert!(result.is_ok());
    assert!(request.param.is_some());

    let Some(param) = request.param else {
        panic!("TestFailed: expected some param value")
    };

    assert_eq!(param["userId"], "userid");
    let mut request_two = Request::new();

    result = router.resolve_path(
        &mut request_two,
        "/api/user/user_one/posts",
        &RequestMethod::Get,
    );
    assert!(result.is_ok());
    let Some(param) = request_two.param else {
        panic!("TestFailed: expected some param value")
    };
    assert_eq!(param["userId"], "user_one");

    let mut request_three = Request::new();

    result = router.resolve_path(
        &mut request_three,
        "/api/user/user_one/post/post_one",
        &RequestMethod::Get,
    );
    assert!(result.is_ok());
    let Some(paramtwo) = request_three.param else {
        panic!("TestFailed: expected some param value")
    };
    assert_eq!(paramtwo["userId"], "user_one");
    assert_eq!(paramtwo["postId"], "post_one");
}

#[test]
fn test_router_resolves_path_with_query() {
    let mut router = Router::new();

    //static rouers
    router.post("/api/user", mock_response).unwrap();
    router.get("/api/users", mock_response).unwrap();

    let mut req = Request::new();
    let result = router.resolve_path(
        &mut req,
        "/api/users?sort=asc&limit=10",
        &RequestMethod::Get,
    );

    assert!(result.is_ok());
    let Some(query) = req.query else {
        panic!("TestFailed: Expected some query values")
    };

    assert_eq!(query["sort"], "asc");
    assert_eq!(query["limit"], "10");
}

#[test]
fn test_router_resolves_static_on_a_dynamic_path() {
    let mut router = Router::new();

    router.post("/api/user", mock_response).unwrap();

    // overlapping routes
    router.post("/api/users/new", mock_response).unwrap();
    router.get("/api/users/:userId", mock_response).unwrap();

    let mut req = Request::new();
    let mut result = router.resolve_path(&mut req, "/api/users/new", &RequestMethod::Post);

    assert!(result.is_ok());
    if let Some(no_param) = req.param {
        assert!(no_param.is_empty());
    }

    let mut request = Request::new();
    result = router.resolve_path(&mut request, "/api/users/user_one", &RequestMethod::Get);

    assert!(result.is_ok());
    let Some(param) = request.param else {
        panic!("TestFailed: Expected some params");
    };
    assert_eq!(param["userId"], "user_one");
}

#[test]
fn test_router_no_match_failure() {
    let mut router = Router::new();

    //static rouers
    router.post("/api/user", mock_response).unwrap();
    router.get("/api/users", mock_response).unwrap();

    let mut req = Request::new();
    let mut result = router.resolve_path(&mut req, "/api/user_x", &RequestMethod::Post);

    assert!(result.is_err());
    let not_found_err_value = result.unwrap_err();
    assert_eq!(not_found_err_value, RouterError::RouteNotFound.to_string());

    result = router.resolve_path(&mut req, "/api/user", &RequestMethod::Patch);

    assert!(result.is_err());
    let method_not_found_err = result.unwrap_err();

    assert_eq!(
        method_not_found_err,
        RouterError::MethodNotFound.to_string()
    );
}

#[test]
fn test_router_trailing_slash() {
    let mut router = Router::new();
    router.post("/api/user", mock_response).unwrap();
    router.get("/api/users/:id", mock_response).unwrap();

    let mut req = Request::new();
    let result = router.resolve_path(&mut req, "/api//", &RequestMethod::Post);

    println!("REsultt ==> {:?}", result);
    assert!(result.is_err());
}

#[test]
fn test_router_out_of_bounds() {
    let mut router = Router::new();

    router
        .post("/api/user/:userId/post", mock_response)
        .unwrap();
    router.get("/api/users/:userId", mock_response).unwrap();
    router
        .get("/api/user/:userId/post/:postId", mock_response)
        .unwrap();

    router.show_routes();

    let mut req = Request::new();
    let result = router.resolve_path(
        &mut req,
        "/api/users/user_one/out/of/bounds",
        &RequestMethod::Get,
    );
    assert!(result.is_err());
    let out_of_bound_err = result.unwrap_err();
    assert_eq!(out_of_bound_err, RouterError::RouteNotFound.to_string());
}
