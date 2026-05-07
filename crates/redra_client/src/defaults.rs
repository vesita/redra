//! 推荐配置、材质色板和 mesh 类型约束
//!
//! Redra 提供三层材质体系：
//! - **基础色** (`base/`) — 7 种感知均匀的基础颜色
//! - **聚类色板** (`data/`) — 12 种最大化区分度的数据可视化颜色
//! - **语义色** (`semantic/`) — 按用途命名的专用颜色

// ============================================================================
// Mesh 约束
// ============================================================================

/// 支持的 mesh 类型及其约束
///
/// | 类型 | 构造方式 | 约束 |
/// |------|---------|------|
/// | Point | `ShapeBuilder::point(x, y, z)` | 无约束，聚合为单次 draw call |
/// | Sphere | `ShapeBuilder::sphere(radius)` | radius > 0 |
/// | Cylinder | `ShapeBuilder::cylinder(radius, height)` | radius > 0, height > 0 |
/// | Cone | `ShapeBuilder::cone(radius, height)` | radius > 0, height > 0 |
/// | Line | `ShapeBuilder::line(...)` | 起终点距离 > 0.001 |
/// | Cube/OBB | `ShapeBuilder::cube(vertices)` | 8 个角点，每个维度 > 0.001 |
pub mod mesh_constraints {
    pub const MIN_CUBE_DIMENSION: f32 = 0.001;
    pub const MIN_LINE_LENGTH: f32 = 0.001;
    pub const MIN_SPHERE_RADIUS: f32 = 0.01;
    pub const MIN_CYLINDER_RADIUS: f32 = 0.005;
    pub const MIN_CYLINDER_HEIGHT: f32 = 0.01;
}

// ============================================================================
// 材质常量
// ============================================================================

/// 基础颜色（`assets/materials/base/`，别名 `"red"` / `"green"` / `"blue"` 等）
///
/// 感知均匀校正后的 7 色色板，各色在暗背景下亮度一致。
///
/// | 名称 | RGB | 用途 |
/// |------|-----|------|
/// | `red` | (0.94, 0.35, 0.32) | 暴力/静态物体、X 轴 |
/// | `green` | (0.30, 0.85, 0.42) | 安全/地面、Y 轴 |
/// | `blue` | (0.42, 0.38, 0.94) | 信息/天空、Z 轴 |
/// | `yellow` | (0.92, 0.74, 0.26) | 警告/高亮 |
/// | `cyan` | (0.26, 0.80, 0.94) | 人物/冷色系 |
/// | `magenta` | (0.72, 0.38, 0.94) | 强调/紫系 |
/// | `white` | (1.0, 1.0, 1.0) | 默认/中性 |
pub mod base {
    pub const RED: &str = "red";
    pub const GREEN: &str = "green";
    pub const BLUE: &str = "blue";
    pub const YELLOW: &str = "yellow";
    pub const CYAN: &str = "cyan";
    pub const MAGENTA: &str = "magenta";
    pub const WHITE: &str = "white";
}

/// 聚类色板（`assets/materials/data/`，别名 `"cluster_01"` ~ `"cluster_12"`）
///
/// 12 种最大化感知区分度的颜色，相邻色相间隔 30°，
/// 统一饱和度 92%、亮度 62%，确保在 3D 暗背景下等亮度、高区分。
///
/// 建议循环使用：`CLUSTER[i % 12]`
///
/// | # | 名称 | 色相 | RGB |
/// |---|------|------|-----|
/// | 01 | 红色 | 4° | (0.94, 0.35, 0.32) |
/// | 02 | 橙色 | 24° | (0.94, 0.56, 0.26) |
/// | 03 | 琥珀色 | 44° | (0.92, 0.74, 0.26) |
/// | 04 | 黄绿色 | 84° | (0.65, 0.85, 0.26) |
/// | 05 | 翠绿色 | 144° | (0.30, 0.85, 0.42) |
/// | 06 | 青绿色 | 168° | (0.26, 0.84, 0.72) |
/// | 07 | 青色 | 190° | (0.26, 0.80, 0.94) |
/// | 08 | 天蓝色 | 220° | (0.30, 0.56, 0.94) |
/// | 09 | 蓝色 | 244° | (0.42, 0.38, 0.94) |
/// | 10 | 紫罗兰 | 274° | (0.72, 0.38, 0.94) |
/// | 11 | 粉红色 | 334° | (0.94, 0.36, 0.62) |
/// | 12 | 玫瑰红 | 350° | (0.94, 0.32, 0.42) |
pub mod cluster {
    pub const C01_RED: &str = "cluster_01";
    pub const C02_ORANGE: &str = "cluster_02";
    pub const C03_AMBER: &str = "cluster_03";
    pub const C04_CHARTREUSE: &str = "cluster_04";
    pub const C05_GREEN: &str = "cluster_05";
    pub const C06_TEAL: &str = "cluster_06";
    pub const C07_CYAN: &str = "cluster_07";
    pub const C08_AZURE: &str = "cluster_08";
    pub const C09_BLUE: &str = "cluster_09";
    pub const C10_VIOLET: &str = "cluster_10";
    pub const C11_PINK: &str = "cluster_11";
    pub const C12_ROSE: &str = "cluster_12";

    /// 全部聚类颜色，按色相排序，适合 `colors[i % colors.len()]` 循环使用
    pub const ALL: &[&str] = &[
        C01_RED, C02_ORANGE, C03_AMBER, C04_CHARTREUSE,
        C05_GREEN, C06_TEAL, C07_CYAN, C08_AZURE,
        C09_BLUE, C10_VIOLET, C11_PINK, C12_ROSE,
    ];
}

/// 语义色（`assets/materials/semantic/`，别名 `"point_cloud"` / `"ground"` 等）
///
/// 按可视化用途命名，每种颜色传达特定含义。
///
/// | 名称 | 用途 | 说明 |
/// |------|------|------|
/// | `point_cloud` | 默认点云 | 暖白色 (0.92, 0.90, 0.82) |
/// | `ground` | 地面点 | 暗橄榄绿，低饱和不抢视线 |
/// | `noise` | 噪声/无效点 | 低饱和灰紫，区别于有效数据 |
/// | `bounding_box` | 包围盒 | 亮绿色，高可见性 |
/// | `trajectory` | 轨迹线 | 亮青色，连续性强 |
/// | `selected` | 选中实体 | 白金色 + 发光 |
/// | `alert` | 警告/异常 | 琥珀色 + 发光 |
/// | `sky` | 天空/背景 | 深灰蓝，不干扰数据 |
pub mod semantic {
    pub const POINT_CLOUD: &str = "point_cloud";
    pub const GROUND: &str = "ground";
    pub const NOISE: &str = "noise";
    pub const BOUNDING_BOX: &str = "bounding_box";
    pub const TRAJECTORY: &str = "trajectory";
    pub const SELECTED: &str = "selected";
    pub const ALERT: &str = "alert";
    pub const SKY: &str = "sky";
}

/// 效果材质（`assets/materials/effects/`，别名 `"glass"` / `"glow"` / `"metal"` 等）
///
/// | 名称 | 效果 |
/// |------|------|
/// | `glass` | 半透明玻璃 (alpha=0.7) |
/// | `glow` | 发光（绿色自发光） |
/// | `metal` | 金属反射 (metallic=0.9) |
/// | `matte` | 哑光 (roughness=0.9) |
/// | `plastic` | 塑料质感 (roughness=0.6) |
/// | `wood` | 木质暖色 |
pub mod effects {
    pub const GLASS: &str = "glass";
    pub const GLOW: &str = "glow";
    pub const METAL: &str = "metal";
    pub const MATTE: &str = "matte";
    pub const PLASTIC: &str = "plastic";
    pub const WOOD: &str = "wood";
}
