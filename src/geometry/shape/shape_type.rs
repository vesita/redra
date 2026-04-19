#[derive(Debug, Clone)]
pub enum ShapeType {
    Point,
    Line,
    Cube,
    Sphere,
    CustomMesh(String),
}