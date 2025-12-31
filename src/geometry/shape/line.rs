use bevy::prelude::*;
use log::info;

use crate::geometry::shape::RDPoint;



pub struct RDSegment {
    pub start: RDPoint,
    pub end: RDPoint,
}

impl RDSegment {
    pub fn to_mesh(&self) -> Mesh {
        info!("RDSegment::to_mesh");
        Mesh::from(Segment3d::new(self.start.to_vec3(), self.end.to_vec3()))
    }
}