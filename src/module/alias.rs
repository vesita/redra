use std::sync::{Arc, Mutex};




pub type ThLc<T> = Arc<Mutex<T>>;
