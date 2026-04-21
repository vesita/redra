use crate::rdmp::{Cone, Cylinder, ExMesh, Line, Point, Sphere, ex_mesh };

impl ExMesh {
    pub fn set_point<T: Into<Point>>(&mut self, point: T) -> Result<(), String> {
        self.u_mesh = Some(ex_mesh::UMesh::Point(point.into()));
        Ok(())
    }

    pub fn set_line<T: Into<Line>>(&mut self, line: T) -> Result<(), String> {
        self.u_mesh = Some(ex_mesh::UMesh::Line(line.into()));
        Ok(())
    }

    pub fn set_sphere<T: Into<Sphere>>(&mut self, sphere: T) -> Result<(), String> {
        self.u_mesh = Some(ex_mesh::UMesh::Sphere(sphere.into()));
        Ok(())
    }

    pub fn set_cylinder<T: Into<Cylinder>>(&mut self, cylinder: T) -> Result<(), String> {
        self.u_mesh = Some(ex_mesh::UMesh::Cylinder(cylinder.into()));
        Ok(())
    }

    pub fn set_cone<T: Into<Cone>>(&mut self, cone: T) -> Result<(), String> {
        self.u_mesh = Some(ex_mesh::UMesh::Cone(cone.into()));
        Ok(())
    }
}

impl From<Point> for ExMesh {
    fn from(point: Point) -> Self {
        ExMesh {
            u_mesh: Some(ex_mesh::UMesh::Point(point)),
        }
    }
}

impl From<Line> for ExMesh {
    fn from(line: Line) -> Self {
        ExMesh {
            u_mesh: Some(ex_mesh::UMesh::Line(line)),
        }
    }
}

impl From<Sphere> for ExMesh {
    fn from(sphere: Sphere) -> Self {
        ExMesh {
            u_mesh: Some(ex_mesh::UMesh::Sphere(sphere)),
        }
    }
}

impl From<Cylinder> for ExMesh {
    fn from(cylinder: Cylinder) -> Self {
        ExMesh {
            u_mesh: Some(ex_mesh::UMesh::Cylinder(cylinder)),
        }
    }
}

impl From<Cone> for ExMesh {
    fn from(cone: Cone) -> Self {
        ExMesh {
            u_mesh: Some(ex_mesh::UMesh::Cone(cone)),
        }
    }
}

// 实现 Into<Point> trait 以便支持更多类型的输入
impl From<(f32, f32, f32)> for Point {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Point { x, y, z }
    }
}

// 实现 Into<Sphere> trait 以便支持更多类型的输入
impl From<(Point, f32)> for Sphere {
    fn from((location, radius): (Point, f32)) -> Self {
        Sphere {
            location: Some(location),
            radius,
        }
    }
}

// 实现 Into<Line> trait 以便支持更多类型的输入
impl From<(Point, Point)> for Line {
    fn from((start, end): (Point, Point)) -> Self {
        Line {
            start: Some(start),
            end: Some(end),
        }
    }
}

// 实现 Into<Cylinder> trait
impl From<(f32, f32)> for Cylinder {
    fn from((radius, height): (f32, f32)) -> Self {
        Cylinder { radius, height }
    }
}

// 实现 Into<Cone> trait
impl From<(f32, f32)> for Cone {
    fn from((radius, height): (f32, f32)) -> Self {
        Cone { radius, height }
    }
}
