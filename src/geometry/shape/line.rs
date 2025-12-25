use bevy::prelude::*;

use crate::geometry::shape::RDPoint;



pub struct RDSegment {
    pub start: RDPoint,
    pub end: RDPoint,
}

impl RDSegment {
    pub fn to_mesh(&self) -> Mesh {
        Mesh::from(Segment3d::new(self.start.to_vec3(), self.end.to_vec3()))
    }
}