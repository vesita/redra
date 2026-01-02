use bevy::{
    camera::{CameraOutputMode, Viewport, visibility::RenderLayers}, 
    prelude::*, 
    render::render_resource::BlendState, 
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_egui::{EguiContext, EguiContexts, EguiGlobalSettings, EguiPrimaryContextPass, PrimaryEguiContext};

use crate::graph::action::clear::ClearAllMessage;

// 定义面板资源，用于存储面板的尺寸信息
#[derive(Resource, Default)]
pub struct PanelState {
    pub left_width: f32,
    pub right_width: f32,
    pub top_height: f32,
}

// 定义面板可见性资源
#[derive(Resource)]
pub struct PanelVisibility {
    pub visible: bool,
}

impl Default for PanelVisibility {
    fn default() -> Self {
        Self { visible: false } // 默认为隐藏，当光标释放时显示
    }
}

// 定义面板插件
pub struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PanelState>()
            .init_resource::<PanelVisibility>()
            .add_message::<ClearAllMessage>()  // 初始化ClearAllMessage消息
            .add_systems(Startup, setup_ui_camera)
            .add_systems(EguiPrimaryContextPass, (ui_panel_system, update_panel_visibility))
            .add_systems(Update, toggle_panel_on_cursor_change);
    }
}

// 设置UI相机
fn setup_ui_camera(
    mut commands: Commands,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
    asset_server: Res<AssetServer>
) {
    // 禁用自动创建主上下文，以便手动设置我们需要的相机
    egui_global_settings.auto_create_primary_context = false;

    let _ = asset_server.load::<Font>("fonts/JetBrainsMapleMono-XX-XX-XX-XX/JetBrainsMapleMono-Light.ttf");

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

// 检测光标状态变化并更新面板可见性
fn toggle_panel_on_cursor_change(
    cursor_options: Single<&CursorOptions>,
    mut panel_visibility: ResMut<PanelVisibility>,
) {
    // 当光标被释放（非锁定状态）时显示面板，当光标被锁定时隐藏面板
    match cursor_options.grab_mode {
        CursorGrabMode::None | CursorGrabMode::Confined => {
            // 光标未锁定或受限，显示面板
            panel_visibility.visible = true;
        }
        CursorGrabMode::Locked => {
            // 光标被锁定，隐藏面板
            panel_visibility.visible = false;
        }
    }
}

// 更新面板可见性的系统
fn update_panel_visibility(
    panel_visibility: Res<PanelVisibility>,
    mut contexts: EguiContexts,
) {
    if !panel_visibility.is_changed() {
        return;
    }

    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,  // 如果无法获取上下文，直接返回
    };

    // 更新egui上下文的显示状态
    ctx.set_pixels_per_point(if panel_visibility.visible { 1.0 } else { 0.1 });
}

// UI面板系统，每帧运行，更新viewport以适应面板
fn ui_panel_system(
    mut contexts: EguiContexts,
    mut panel_state: ResMut<PanelState>,
    panel_visibility: Res<PanelVisibility>,
    mut camera: Query<&mut Camera, (With<PrimaryEguiContext>, Without<EguiContext>)>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut clear_message: MessageWriter<ClearAllMessage>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,  // 如果无法获取上下文，直接返回
    };

    let Ok(window) = window.single() else {
        return;
    };

    // 如果面板不可见，直接返回
    if !panel_visibility.visible {
        return;
    }

    // 创建可调整大小的边方面板
    let mut left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .default_width(200.0)
        .min_width(100.0)
        .show(&ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("左侧边栏");
                ui.separator();
                
                ui.collapsing("场景控制", |ui| {
                    ui.label("控制场景的参数");
                    let _ = ui.button("重置视角");
                    let _ = ui.button("切换视角");
                });
                
                ui.collapsing("对象列表", |ui| {
                    if ui.button("clear all").clicked() {
                        info!("发送清除所有对象消息");
                        clear_message.write(ClearAllMessage);
                    }
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
        .show(&ctx, |ui| {
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
        .show(&ctx, |ui| {
            egui::Frame::new().show(ui, |ui| {
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
                    ui.heading("Redra - 3D可视化系统"); // 使用heading控件，它会应用全局字体设置
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

    // 将尺寸从逻辑单位转换为物理单位
    left *= window.scale_factor();
    right *= window.scale_factor();
    top *= window.scale_factor();

    // 计算主视口位置和尺寸
    let pos = UVec2::new(left as u32, top as u32);
    let size = UVec2::new(
        (window.physical_width() as f32 - left - right) as u32,
        (window.physical_height() as f32 - top) as u32,
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