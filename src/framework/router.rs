use std::collections::HashMap;

use crate::internal::{request::Request, response::Response};

pub type Handler = fn(&Request) -> Response;

