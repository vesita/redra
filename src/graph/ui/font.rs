use std::sync::Arc;

use bevy_egui::EguiContexts;

pub fn replace_fonts(
    mut contexts: EguiContexts,
) {
    // 从默认字体开始（我们将添加到它们中，而不是替换它们）
    let mut fonts = egui::FontDefinitions::default();
    
    // 加载自定义字体文件 - 使用实际的字体文件路径，替换占位符为实际的字体文件
    fonts.font_data.insert(
        "jmm".to_owned(),  // JetBrains Maple Mono
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/JetBrainsMapleMono-XX-XX-XX-XX/JetBrainsMapleMono-Regular.ttf"
        ))),
    );
    
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
    
    // 正确处理Result类型 - 使用get_mut()方法安全解包获取egui上下文
    if let Ok(ctx) = contexts.ctx_mut() {
        ctx.set_fonts(fonts);
    }
}