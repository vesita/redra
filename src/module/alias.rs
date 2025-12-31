use std::sync::Arc;
use tokio::sync::Mutex;




pub type ThLc<T> = Arc<Mutex<T>>;