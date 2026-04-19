use crate::rdmp::{Line, Mesh, Point, Sphere, mesh::mesh::AMesh};

impl Mesh {
    pub fn set_point<T: Into<Point>>(&mut self, point: T) -> Result<(), String> {
        self.a_mesh = Some(AMesh::Point(point.into()));
        Ok(())
    }

    pub fn set_line<T: Into<Line>>(&mut self, line: T) -> Result<(), String> {
        self.a_mesh = Some(AMesh::Line(line.into()));
        Ok(())
    }

    pub fn set_sphere<T: Into<Sphere>>(&mut self, sphere: T) -> Result<(), String> {
        self.a_mesh = Some(AMesh::Sphere(sphere.into()));
        Ok(())
    }
}

impl From<Point> for Mesh {
    fn from(point: Point) -> Self {
        Mesh {
            a_mesh: Some(AMesh::Point(point)),
        }
    }
}

impl From<Line> for Mesh {
    fn from(line: Line) -> Self {
        Mesh {
            a_mesh: Some(AMesh::Line(line)),
        }
    }
}

impl From<Sphere> for Mesh {
    fn from(sphere: Sphere) -> Self {
        Mesh {
            a_mesh: Some(AMesh::Sphere(sphere)),
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
