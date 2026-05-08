use std::collections::{HashMap, HashSet};
use bevy::prelude::*;
use expto::rdmp::{TagCollectionDef, TagOption};

// ==================== Tag 集合注册表 ====================

/// Tag 集合定义 — 带显示名称和可选值的分类体系
#[derive(Clone, Debug)]
pub struct TagCollection {
    pub name: String,
    pub display_name: String,
    pub options: Vec<TagOption>,
    pub source: CollectionSource,
}

#[derive(Clone, Debug)]
pub enum CollectionSource {
    Static(String),   // 来自 TOML 文件
    Dynamic,          // 通过网络传输
}

/// Bevy Resource：全局 Tag 集合定义注册表
#[derive(Resource, Default)]
pub struct TagRegistry {
    pub collections: HashMap<String, TagCollection>,
}

impl TagRegistry {
    pub fn register(&mut self, def: TagCollectionDef, source: CollectionSource) {
        let collection = TagCollection {
            name: def.name.clone(),
            display_name: def.display_name,
            options: def.options,
            source,
        };
        log::info!("注册 Tag 集合: {}", collection.name);
        self.collections.insert(collection.name.clone(), collection);
    }

    pub fn get(&self, name: &str) -> Option<&TagCollection> {
        self.collections.get(name)
    }

    /// 从 TOML 配置加载
    pub fn load_static(&mut self, config_path: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(config_path)
            .map_err(|e| format!("读取 Tag 配置失败: {}", e))?;
        let defs: Vec<TagCollectionDef> = toml::from_str(&content)
            .map_err(|e| format!("TOML 解析失败: {}", e))?;
        for def in defs {
            self.register(def, CollectionSource::Static(config_path.to_string()));
        }
        Ok(())
    }
}

// ==================== Tag 筛选 ====================

/// 筛选模式
#[derive(Clone, Debug, PartialEq)]
pub enum FilterMode {
    /// 不筛选，显示全部
    PassThrough,
    /// 简单模式：按集合+值精确筛选
    Simple,
    /// 复杂模式：布尔表达式
    Advanced,
}

impl Default for FilterMode {
    fn default() -> Self { Self::PassThrough }
}

/// 简单模式的一条规则：指定集合中允许哪些值
#[derive(Clone, Debug)]
pub struct SimpleFilterRule {
    pub collection: String,
    pub allowed_values: HashSet<String>,
}

/// 复杂模式表达式
#[derive(Clone, Debug)]
pub enum FilterExpr {
    ShowIf { collection: String, value: String },
    HideIf { collection: String, value: String },
    And(Box<FilterExpr>, Box<FilterExpr>),
    Or(Box<FilterExpr>, Box<FilterExpr>),
    Not(Box<FilterExpr>),
}

/// Bevy Resource：渲染筛选规则
#[derive(Resource)]
pub struct TagFilter {
    pub enabled: bool,
    pub mode: FilterMode,
    pub simple_rules: Vec<SimpleFilterRule>,
    pub expression: Option<FilterExpr>,
    /// 高级模式文本（UI 编辑用）
    pub advanced_text: String,
}

impl Default for TagFilter {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: FilterMode::PassThrough,
            simple_rules: Vec::new(),
            expression: None,
            advanced_text: String::new(),
        }
    }
}

impl TagFilter {
    /// 从简单规则切换为复杂模式
    pub fn switch_to_advanced(&mut self) {
        self.mode = FilterMode::Advanced;
    }

    /// 从复杂模式切换回简单模式
    pub fn switch_to_simple(&mut self) {
        self.mode = FilterMode::Simple;
    }

    /// 启用/停用筛选
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// 解析高级表达式文本
    pub fn parse_expression(&mut self, text: &str) -> Result<(), String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            self.expression = None;
            return Ok(());
        }
        self.expression = Some(Self::parse_expr(trimmed)?);
        Ok(())
    }

    fn parse_expr(s: &str) -> Result<FilterExpr, String> {
        let s = s.trim();

        // 处理 NOT: !(...)
        if let Some(rest) = s.strip_prefix('!') {
            return Ok(FilterExpr::Not(Box::new(Self::parse_expr(rest)?)));
        }

        // 处理 OR: 在顶层按 ` OR ` 分割
        if let Some(pos) = Self::find_op_at_top_level(s, " OR ") {
            let left = Self::parse_expr(&s[..pos])?;
            let right = Self::parse_expr(&s[pos + 4..])?;
            return Ok(FilterExpr::Or(Box::new(left), Box::new(right)));
        }

        // 处理 AND: 在顶层按 ` AND ` 分割
        if let Some(pos) = Self::find_op_at_top_level(s, " AND ") {
            let left = Self::parse_expr(&s[..pos])?;
            let right = Self::parse_expr(&s[pos + 5..])?;
            return Ok(FilterExpr::And(Box::new(left), Box::new(right)));
        }

        // 处理括号: (...)
        if s.starts_with('(') && s.ends_with(')') {
            return Self::parse_expr(&s[1..s.len()-1]);
        }

        // 处理原子: collection=value
        if let Some(eq_pos) = s.find('=') {
            let collection = s[..eq_pos].trim().to_string();
            let value = s[eq_pos + 1..].trim().to_string();
            if collection.is_empty() || value.is_empty() {
                return Err(format!("无效表达式: '{}'", s));
            }
            return Ok(FilterExpr::ShowIf { collection, value });
        }

        Err(format!("无法解析表达式: '{}'", s))
    }

    /// 在顶层（括号外）查找运算符
    fn find_op_at_top_level(s: &str, op: &str) -> Option<usize> {
        let mut depth = 0;
        let mut chars = s.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            if depth == 0 && s[i..].starts_with(op) {
                return Some(i);
            }
        }
        None
    }
}

// ==================== Tag 筛选评估 ====================

/// 判断一个实体的 tag 列表是否通过当前筛选
pub fn entity_passes_filter(
    tags: &[expto::rdmp::Tag],
    filter: &TagFilter,
    _registry: &TagRegistry,
) -> bool {
    if !filter.enabled {
        return true;
    }

    match &filter.mode {
        FilterMode::PassThrough => true,
        FilterMode::Simple => simple_filter_check(tags, filter),
        FilterMode::Advanced => advanced_filter_check(tags, filter),
    }
}

/// 简单模式：所有 simple_rules 下的 allowed_values 包含实体的 tag 值
fn simple_filter_check(tags: &[expto::rdmp::Tag], filter: &TagFilter) -> bool {
    if filter.simple_rules.is_empty() {
        return true; // 没有规则 = 全部显示
    }

    for rule in &filter.simple_rules {
        // 从 tags 中找出对应 collection 的 tag
        let prefix = format!("{}:", rule.collection);
        let tag_value = tags.iter()
            .find(|t| t.text.starts_with(&prefix))
            .map(|t| t.text[prefix.len()..].to_string());

        match tag_value {
            Some(val) => {
                if !rule.allowed_values.contains(&val) {
                    return false; // 该集合的 tag 值不在允许列表中 → 隐藏
                }
            }
            None => {
                // 实体没有该集合的 tag
                // 如果集合的 allowed_values 为空（全选），则通过
                if !rule.allowed_values.is_empty() {
                    return false; // 集合有筛选但实体没该 tag → 隐藏
                }
            }
        }
    }

    true
}

/// 复杂模式：递归评估 FilterExpr
fn advanced_filter_check(tags: &[expto::rdmp::Tag], filter: &TagFilter) -> bool {
    let Some(expr) = &filter.expression else {
        return true;
    };
    eval_expr(tags, expr)
}

fn eval_expr(tags: &[expto::rdmp::Tag], expr: &FilterExpr) -> bool {
    match expr {
        FilterExpr::ShowIf { collection, value } => {
            let prefix = format!("{}:", collection);
            tags.iter().any(|t| t.text == *value || t.text == format!("{}:{}", collection, value) || (t.text.starts_with(&prefix) && t.text[prefix.len()..] == *value))
        }
        FilterExpr::HideIf { collection, value } => {
            !eval_expr(tags, &FilterExpr::ShowIf { collection: collection.clone(), value: value.clone() })
        }
        FilterExpr::And(a, b) => eval_expr(tags, a) && eval_expr(tags, b),
        FilterExpr::Or(a, b) => eval_expr(tags, a) || eval_expr(tags, b),
        FilterExpr::Not(inner) => !eval_expr(tags, inner),
    }
}
