use lb::AllowedMethods;
use lb::Request;
use lb::Response;
use lb::Router;

fn mock_response(_req: &Request) -> Response {
    Response::new(200, "OK", None)
}

#[test]
fn test_router_resolves_static_path() {
    let mut router = Router::new();

    router.get("/api/v1/dashboard", mock_response).unwrap();

    let mut mock_req = Request::new();

    let result = router.resolve_path(&mut mock_req, "/api/v1/dashboard", AllowedMethods::GET);

    assert!(
        result.is_ok(),
        "Expected an Ok result but got {:?}",
        result.err()
    );

    assert!(mock_req.param.is_none());
    assert!(mock_req.query.is_none());

    let resolved_handler = result.unwrap();
    let res = resolved_handler(&mock_req);
    println!("res => {:?}", res);
    assert_eq!(res.status_code, 200);
}

#[test]
fn test_router_resolves_dynamic_path() {
    let mut router = Router::new();
    router.post("/api/v1/user/:id", mock_response).unwrap();
    let mut mock_req = Request::new();

    router.show_routes();
    println!("About to resolve path");
    let result = router.resolve_path(&mut mock_req, "/api/v1/user/usserid/", AllowedMethods::POST);

    println!("errror => {:?}", result);

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

    router.show_routes();
    let mut request = Request::new();

    let mut result = router.resolve_path(&mut request, "/api/user", AllowedMethods::POST);
    assert!(result.is_ok());
    result = router.resolve_path(&mut request, "/api/users", AllowedMethods::GET);
    println!("result for apiusers => {:?}", result);
    assert!(result.is_ok());
    result = router.resolve_path(&mut request, "/api/user/userid/post", AllowedMethods::POST);
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
        AllowedMethods::GET,
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
        AllowedMethods::GET,
    );
    println!("REsult should fail => {:?}", result);
    assert!(result.is_ok());
    let Some(paramtwo) = request_three.param else {
        panic!("TestFailed: expected some param value")
    };
    assert_eq!(paramtwo["userId"], "user_one");
    assert_eq!(paramtwo["postId"], "post_one");
}           
