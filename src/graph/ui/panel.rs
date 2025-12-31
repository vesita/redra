use bevy::{
    camera::{CameraOutputMode, Viewport, visibility::RenderLayers}, 
    prelude::*, 
    render::render_resource::BlendState, 
    window::PrimaryWindow
};
use bevy_egui::{
    EguiContext, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass,
    PrimaryEguiContext, egui,
};

// 定义面板资源，用于存储面板的尺寸信息
#[derive(Resource, Default)]
pub struct PanelState {
    pub left_width: f32,
    pub right_width: f32,
    pub top_height: f32,
    pub bottom_height: f32,
}

// 定义面板插件
pub struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PanelState>()
            .add_systems(Startup, setup_ui_camera)
            .add_systems(Update, ui_panel_system);
    }
}

// 设置UI相机
fn setup_ui_camera(
    mut commands: Commands,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
) {
    // 禁用自动创建主上下文，以便手动设置我们需要的相机
    egui_global_settings.auto_create_primary_context = false;

    // 主世界相机
    commands.spawn((
        Camera2d,
        Name::new("Main Camera")
    ));

    // EGUI相机，用于渲染UI
    commands.spawn((
        // PrimaryEguiContext组件需要渲染主上下文的所有内容
        PrimaryEguiContext,
        Camera2d::default(),
        // 设置渲染层为无，确保我们只渲染UI
        RenderLayers::none(),
        Camera {
            order: 2,  // 设置更高的渲染顺序，确保UI在主相机之上渲染
            output_mode: CameraOutputMode::Write {
                blend_state: Some(BlendState::ALPHA_BLENDING),
                clear_color: ClearColorConfig::None,
            },
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        Name::new("UI Camera")
    ));
}

// UI面板系统，每帧运行，更新viewport以适应面板
fn ui_panel_system(
    mut contexts: EguiContexts,
    mut panel_state: ResMut<PanelState>,
    mut camera: Query<&mut Camera, (With<PrimaryEguiContext>, Without<EguiContext>)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let ctx = contexts.ctx_mut();
    let ctx = match ctx {
        Ok(ctx) => ctx,
        Err(_) => return,
    };
    
    if !ctx.wants_pointer_input() {
        return;
    }

    let Ok(window) = window.single() else {
        return;
    };

    // 创建可调整大小的边方面板
    let mut left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .default_width(200.0)
        .min_width(100.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("左侧边栏");
                ui.separator();
                
                ui.collapsing("场景控制", |ui| {
                    ui.label("控制场景的参数");
                    ui.button("重置视角");
                    ui.button("切换视角");
                });
                
                ui.collapsing("对象列表", |ui| {
                    ui.selectable_value(&mut 0, 0, "对象 1");
                    ui.selectable_value(&mut 1, 1, "对象 2");
                    ui.selectable_value(&mut 2, 2, "对象 3");
                });
                
                ui.collapsing("设置", |ui| {
                    ui.checkbox(&mut true, "显示网格");
                    ui.checkbox(&mut true, "显示坐标轴");
                    ui.checkbox(&mut false, "显示边界框");
                });
            });
        })
        .response
        .rect
        .width();

    let mut right = egui::SidePanel::right("right_panel")
        .resizable(true)
        .default_width(250.0)
        .min_width(150.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("右侧边栏");
                ui.separator();
                
                ui.collapsing("属性编辑器", |ui| {
                    ui.label("位置");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut 0.0).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut 0.0).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut 0.0).speed(0.1));
                    });
                });
                
                ui.collapsing("材质", |ui| {
                    ui.label("材质类型");
                    ui.radio_value(&mut 0, 0, "基础颜色");
                    ui.radio_value(&mut 1, 1, "发光材质");
                    ui.radio_value(&mut 2, 2, "金属材质");
                });
            });
        })
        .response
        .rect
        .width();

    let mut top = egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .default_height(40.0)
        .min_height(30.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("新建").clicked() {
                        ui.close();
                    }
                    if ui.button("打开").clicked() {
                        ui.close();
                    }
                    if ui.button("保存").clicked() {
                        ui.close();
                    }
                });
                ui.menu_button("编辑", |ui| {
                    if ui.button("撤销").clicked() {
                        ui.close();
                    }
                    if ui.button("重做").clicked() {
                        ui.close();
                    }
                    if ui.button("复制").clicked() {
                        ui.close();
                    }
                });
                ui.menu_button("视图", |ui| {
                    ui.checkbox(&mut true, "显示网格");
                    ui.checkbox(&mut true, "显示坐标轴");
                });
                ui.separator();
                ui.label("Redra - 3D可视化系统");
            });
        })
        .response
        .rect
        .height();

    let mut bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .default_height(100.0)
        .min_height(50.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("状态信息");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("FPS: {:.1}", 60.0));
                    ui.separator();
                    ui.label(format!("对象数: {}", 10));
                    ui.separator();
                    ui.label(format!("内存: {:.1} MB", 128.5));
                });
            });
        })
        .response
        .rect
        .height();

    // 保存面板尺寸到资源中
    panel_state.left_width = left;
    panel_state.right_width = right;
    panel_state.top_height = top;
    panel_state.bottom_height = bottom;

    // 将尺寸从逻辑单位转换为物理单位
    left *= window.scale_factor();
    right *= window.scale_factor();
    top *= window.scale_factor();
    bottom *= window.scale_factor();

    // 计算主视口位置和尺寸
    let pos = UVec2::new(left as u32, top as u32);
    let size = UVec2::new(
        (window.physical_width() as f32 - left - right) as u32,
        (window.physical_height() as f32 - top - bottom) as u32,
    );

    // 更新相机视口
    if let Ok(mut camera) = camera.single_mut() {
        camera.viewport = Some(Viewport {
            physical_position: pos,
            physical_size: size,
            ..default()
        });
    }
}