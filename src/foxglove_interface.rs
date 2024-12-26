use std::sync::{Arc, Mutex};

use crate::foxglove_state::FoxgloveState;

#[derive(Clone)]
pub struct FoxgloveInterface {
    pub state: Arc<Mutex<FoxgloveState>>,
}
