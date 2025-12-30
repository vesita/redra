use std::collections::HashMap;

use bevy::{asset::Handle, pbr::StandardMaterial, prelude::Resource};
use crate::{ThLc, module::resource::channel::RDChannel};



pub mod channel;
pub mod handle;

#[derive(Resource)]
pub struct RDResource {
    pub channel: RDChannel,
    // pub handle: ThLc<RDHandle>,
    pub materials: HashMap<String, Handle<StandardMaterial>>,
}

impl RDResource {
    pub fn new(
        channel: RDChannel,
        // handle: ThLc<RDHandle>,
    ) -> RDResource {
        RDResource {
            channel,
            // handle,
            materials: HashMap::new(),
        }
    }
}