use bevy::{prelude::*, render::*};

use log::info;
use std::time::Instant;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
// 添加一个资源来跟踪当前的帧率设置
#[derive(Resource, Debug, Clone, Copy)]
pub struct FrameRateState {
    pub change: bool,
    pub frame_rate: f64,
}

pub fn toggle_frame_rate(
    settings: Option<ResMut<FramepaceSettings>>,
    mut frame_rate_state: ResMut<FrameRateState>,
) {
    if frame_rate_state.change == false {
        return;
    }
    
    if let Some(mut settings) = settings {
        settings.limiter = Limiter::from_framerate(frame_rate_state.frame_rate);
        info!("切换帧率到: {} FPS", frame_rate_state.frame_rate);
    }
    
    frame_rate_state.change = false;
}

/// Adds frame-pacing and frame-limiting functionality to your [`App`].
#[derive(Debug, Clone, Component)]
pub struct FramepacePlugin;
impl Plugin for FramepacePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FramepaceSettings>();

        let limit = FrametimeLimit::default();
        let settings = FramepaceSettings::default();
        let settings_proxy = FramepaceSettingsProxy::default();
        let stats = FramePaceStats::default();

        app.insert_resource(settings)
            .insert_resource(settings_proxy.clone())
            .insert_resource(limit.clone())
            .insert_resource(stats.clone())
            .add_systems(Update, update_proxy_resources);

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, get_display_refresh_rate);

        app.sub_app_mut(RenderApp)
            .insert_resource(FrameTimer::default())
            .insert_resource(settings_proxy)
            .insert_resource(limit)
            .insert_resource(stats)
            .add_systems(
                Render,
                framerate_limiter
                    .in_set(RenderSystems::Cleanup)
                    .after(World::clear_entities),
            );
    }
}

/// Framepacing plugin configuration.
#[derive(Debug, Clone, Resource, Reflect)]
#[reflect(Resource)]
pub struct FramepaceSettings {
    /// Configures the framerate limiting strategy.
    pub limiter: Limiter,
}
impl FramepaceSettings {
    /// Builds plugin settings with the specified [`Limiter`] configuration.
    pub fn with_limiter(mut self, limiter: Limiter) -> Self {
        self.limiter = limiter;
        self
    }
}
impl Default for FramepaceSettings {
    fn default() -> FramepaceSettings {
        FramepaceSettings {
            limiter: Limiter::Auto,
        }
    }
}

#[derive(Default, Debug, Clone, Resource)]
struct FramepaceSettingsProxy {
    /// Configures the framerate limiting strategy.
    limiter: Arc<Mutex<Limiter>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl FramepaceSettingsProxy {
    fn is_enabled(&self) -> bool {
        self.limiter.try_lock().iter().any(|l| l.is_enabled())
    }
}

fn update_proxy_resources(settings: Res<FramepaceSettings>, proxy: Res<FramepaceSettingsProxy>) {
    if settings.is_changed()
        && let Ok(mut limiter) = proxy.limiter.try_lock()
    {
        *limiter = settings.limiter.clone();
    }
}

/// Configures the framelimiting technique for the app.
#[derive(Debug, Default, Clone, Reflect)]
pub enum Limiter {
    /// Uses the window's refresh rate to set the frametime limit, updating when the window changes
    /// monitors.
    #[default]
    Auto,
    /// Set a fixed manual frametime limit. This should be greater than the monitors frametime
    /// (`1.0 / monitor frequency`).
    Manual(Duration),
    /// Disables frame limiting
    Off,
}

impl Limiter {
    /// Returns `true` if the [`Limiter`] is enabled.
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Limiter::Off)
    }

    /// Constructs a new [`Limiter`] from the provided `framerate`.
    pub fn from_framerate(framerate: f64) -> Self {
        Limiter::Manual(Duration::from_secs_f64(1.0 / framerate))
    }
}

impl std::fmt::Display for Limiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Limiter::Auto => write!(f, "Auto"),
            Limiter::Manual(t) => write!(f, "{:.2} fps", 1.0 / t.as_secs_f32()),
            Limiter::Off => write!(f, "Off"),
        }
    }
}

/// Current frametime limit based on settings and monitor refresh rate.
#[derive(Debug, Default, Clone, Resource)]
pub struct FrametimeLimit(pub Arc<Mutex<Duration>>);

/// Tracks the instant of the end of the previous frame.
#[derive(Debug, Clone, Resource, Reflect)]
pub struct FrameTimer {
    sleep_end: Instant,
}
impl Default for FrameTimer {
    fn default() -> Self {
        FrameTimer {
            sleep_end: Instant::now(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn get_display_refresh_rate(
    settings: Res<FramepaceSettings>,
    frame_limit: Res<FrametimeLimit>,
) {
    let new_frametime = match settings.limiter {
        Limiter::Auto => {
            // 无法检测实际刷新率，使用默认 60Hz
            Duration::from_secs_f64(1.0 / 60.0)
        },
        Limiter::Manual(frametime) => frametime,
        Limiter::Off => {
            return;
        }
    };

    if let Ok(mut limit) = frame_limit.0.try_lock()
        && new_frametime != *limit
    {
        *limit = new_frametime;
    }
}

/// Holds frame time measurements for framepacing diagnostics
#[derive(Clone, Debug, Default, Resource)]
pub struct FramePaceStats {
    frametime: Arc<Mutex<Duration>>,
    oversleep: Arc<Mutex<Duration>>,
}

/// Accurately sleeps until it's time to start the next frame.
///
/// The `spin_sleep` dependency makes it possible to get extremely accurate sleep times across
/// platforms. Using `std::thread::sleep()` will not be precise enough, especially windows. Using a
/// spin lock, even with `std::hint::spin_loop()`, will result in significant power usage.
///
/// `spin_sleep` sleeps as long as possible given the platform's sleep accuracy, and spins for the
/// remainder. The dependency is however not WASM compatible, which is fine, because frame limiting
/// should not be used in a browser; this would compete with the browser's frame limiter.
#[allow(unused_variables)]
fn framerate_limiter(
    mut timer: ResMut<FrameTimer>,
    target_frametime: Res<FrametimeLimit>,
    stats: Res<FramePaceStats>,
    settings: Res<FramepaceSettingsProxy>,
) {
    if let Ok(limit) = target_frametime.0.try_lock() {
        let frame_time = timer.sleep_end.elapsed();
        #[cfg(not(target_arch = "wasm32"))]
        {
            let oversleep = stats
                .oversleep
                .try_lock()
                .as_deref()
                .cloned()
                .unwrap_or_default();
            let sleep_time = limit.saturating_sub(frame_time + oversleep);
            if settings.is_enabled() {
                spin_sleep::sleep(sleep_time);
            }
        }

        let frame_time_total = timer.sleep_end.elapsed();
        timer.sleep_end = Instant::now();
        if let Ok(mut frametime) = stats.frametime.try_lock() {
            *frametime = frame_time;
        }
        if let Ok(mut oversleep) = stats.oversleep.try_lock() {
            *oversleep = frame_time_total.saturating_sub(*limit);
        }
    };
}