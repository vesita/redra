
use std::collections::HashMap;
use tokio;

use crate::{ThLc, net::{listener::RDListener, server::RDServer}};

pub struct RDHandle {
    pub listener: ThLc<RDListener>,
    pub servers: HashMap<String, ThLc<RDServer>>,
}