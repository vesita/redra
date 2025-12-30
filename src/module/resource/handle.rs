
use std::collections::HashMap;

use crate::{ThLc, net::{listener::RDListener, forwarder::RDForwarder}};

pub struct RDHandle {
    pub listener: ThLc<RDListener>,
    pub servers: HashMap<String, ThLc<RDForwarder>>,
}