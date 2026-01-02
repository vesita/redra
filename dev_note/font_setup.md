# 字体设置开发指南

## 概述

本指南介绍如何在项目中正确设置字体，特别是egui字体系统。字体设置需要在Bevy的启动系统中完成，以确保在UI渲染之前字体已经正确加载。

## 核心原则

### 1. 硬编码字体路径

字体文件必须通过硬编码方式嵌入到二进制文件中，使用`include_bytes!`宏：

```rust
fonts.font_data.insert(
    "jmm".to_owned(),  // 字体ID
    Arc::new(egui::FontData::from_static(include_bytes!(
        "../../../assets/fonts/JetBrainsMapleMono-XX-XX-XX-XX/JetBrainsMapleMono-Regular.ttf"
    ))),
);
```

### 2. 字体优先级设置

通过在字体族数组中的位置来设置字体优先级：

- 使用`insert(0, font_name)`将字体设置为最高优先级
- 使用`push(font_name)`将字体添加到最低优先级

```rust
// 将自定义字体设置为比例字体族的最高优先级（主要字体）
fonts
    .families
    .entry(egui::FontFamily::Proportional)
    .or_default()
    .insert(0, "jmm".to_owned());

// 将自定义字体设置为等宽字体族的后备选项
fonts
    .families
    .entry(egui::FontFamily::Monospace)
    .or_default()
    .push("jmm".to_owned());
```

## 系统注册

### 1. 将字体加载添加到系统中

字体加载函数必须作为Startup系统注册，确保只执行一次：

```rust
pub struct UiModule;

impl Plugin for UiModule {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(EguiPlugin::default())
            .add_plugins(PanelPlugin)
            .add_systems(Startup, replace_fonts);  // 注册字体替换系统
    }
}
```

## 字体文件管理

### 1.  特性标志

使用特性标志来控制不同字体的加载：

```rust
#[cfg(feature = "jetbrains_mono")]
{
    fonts.font_data.insert(
        "jmm".to_owned(),  // JetBrains Maple Mono
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/JetBrainsMapleMono-Regular.ttf"
        ))),
    );
}

#[cfg(feature = "chinese_font")]
{
    fonts.font_data.insert(
        "noto_sans_sc".to_owned(),  // Noto Sans SC 中文字体
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/NotoSansSC-Regular.ttf"
        ))),
    );
}
```

## 注意事项

1. 字体加载系统应该在`EguiPrimaryContextPass`阶段执行，确保egui上下文可用
2. 必须在Bevy 0.14+中使用`add_systems`方法注册系统
3. 避免在运行时动态加载字体文件，应使用编译时嵌入的字体数据
4. 对于中文支持，需要专门的中文字体文件
5. 字体设置操作必须在egui上下文准备就绪后执行

## 完整示例

```rust
use std::sync::Arc;
use bevy_egui::EguiContexts;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct FontLoadState {
    pub loaded: bool,
}

pub fn replace_fonts(
    mut contexts: EguiContexts,
    mut font_load_state: ResMut<FontLoadState>,
) {
    // 检查是否已经加载过字体，避免重复加载
    if font_load_state.loaded {
        return;
    }

    let mut fonts = egui::FontDefinitions::default();
    
    // 嵌入静态字体文件
    fonts.font_data.insert(
        "jetbrains_mono".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/JetBrainsMapleMono-Regular.ttf"
        ))),
    );
    
    // 设置字体优先级
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "jetbrains_mono".to_owned());
        
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "jetbrains_mono".to_owned());

    // 应用字体设置
    contexts.ctx_mut().set_fonts(fonts);

    // 标记字体已加载
    font_load_state.loaded = true;
}
```
