//! 相机控制状态机
//!
//! 提供统一的光标锁定和相机启用状态管理，无需依赖外部 crate。
//!
//! ## 使用示例
//!
//! ```rust
//! use smooth_bevy_cameras::CameraStateMachine;
//!
//! let mut state_machine = CameraStateMachine::new(false, false);
//!
//! // 启用相机并锁定光标（FPS模式）
//! state_machine.enable_with_cursor_locked();
//! assert!(state_machine.is_fps_mode());
//!
//! // 切换光标锁定
//! state_machine.toggle_cursor_lock();
//! assert!(!state_machine.is_fps_mode());
//!
//! // 禁用相机
//! state_machine.disable();
//! assert!(!state_machine.is_enabled());
//! ```

/// 相机控制状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraControlState {
    /// 相机禁用状态
    #[default]
    Disabled,
    /// 相机启用 + 光标锁定（FPS模式）
    EnabledCursorLocked,
    /// 相机启用 + 光标未锁定（编辑模式）
    EnabledCursorUnlocked,
}

impl CameraControlState {
    /// 检查是否处于 FPS 模式（光标锁定）
    pub fn is_fps_mode(&self) -> bool {
        matches!(self, CameraControlState::EnabledCursorLocked)
    }

    /// 检查相机是否启用
    pub fn is_enabled(&self) -> bool {
        !matches!(self, CameraControlState::Disabled)
    }

    /// 获取状态的文本描述
    pub fn as_str(&self) -> &'static str {
        match self {
            CameraControlState::Disabled => "Disabled",
            CameraControlState::EnabledCursorLocked => "Enabled (FPS Mode)",
            CameraControlState::EnabledCursorUnlocked => "Enabled (Edit Mode)",
        }
    }
}

impl std::fmt::Display for CameraControlState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 相机状态变化事件数据
#[derive(Debug, Clone, Copy)]
pub struct CameraStateChanged {
    /// 之前的状态
    pub previous: CameraControlState,
    /// 当前状态
    pub current: CameraControlState,
}

impl CameraStateChanged {
    /// 创建新的事件
    pub fn new(previous: CameraControlState, current: CameraControlState) -> Self {
        Self { previous, current }
    }
}

/// 相机状态机
///
/// 管理相机的启用/禁用和光标锁定状态，提供清晰的状态转换 API。
#[derive(Debug, Clone, Copy)]
pub struct CameraStateMachine {
    current_state: CameraControlState,
}

impl Default for CameraStateMachine {
    fn default() -> Self {
        Self::new(false, false)
    }
}

impl CameraStateMachine {
    /// 创建新的状态机
    ///
    /// # 参数
    /// * `enabled` - 相机是否启用
    /// * `cursor_locked` - 光标是否锁定
    pub fn new(enabled: bool, cursor_locked: bool) -> Self {
        let current_state = if !enabled {
            CameraControlState::Disabled
        } else if cursor_locked {
            CameraControlState::EnabledCursorLocked
        } else {
            CameraControlState::EnabledCursorUnlocked
        };

        Self { current_state }
    }

    /// 获取当前状态
    pub fn current_state(&self) -> CameraControlState {
        self.current_state
    }

    /// 检查是否处于 FPS 模式
    pub fn is_fps_mode(&self) -> bool {
        self.current_state.is_fps_mode()
    }

    /// 检查相机是否启用
    pub fn is_enabled(&self) -> bool {
        self.current_state.is_enabled()
    }

    /// 检查光标是否锁定
    pub fn is_cursor_locked(&self) -> bool {
        matches!(self.current_state, CameraControlState::EnabledCursorLocked)
    }

    /// 启用相机并保持当前光标状态
    pub fn enable(&mut self) -> Option<CameraStateChanged> {
        if self.current_state.is_enabled() {
            return None; // 已经启用
        }

        let previous = self.current_state;
        // 默认启用时不锁定光标（安全默认值）
        self.current_state = CameraControlState::EnabledCursorUnlocked;

        Some(CameraStateChanged::new(previous, self.current_state))
    }

    /// 启用相机并锁定光标（进入 FPS 模式）
    pub fn enable_with_cursor_locked(&mut self) -> Option<CameraStateChanged> {
        if self.current_state == CameraControlState::EnabledCursorLocked {
            return None; // 已经是该状态
        }

        let previous = self.current_state;
        self.current_state = CameraControlState::EnabledCursorLocked;

        Some(CameraStateChanged::new(previous, self.current_state))
    }

    /// 启用相机但不锁定光标（编辑模式）
    pub fn enable_with_cursor_unlocked(&mut self) -> Option<CameraStateChanged> {
        if self.current_state == CameraControlState::EnabledCursorUnlocked {
            return None; // 已经是该状态
        }

        let previous = self.current_state;
        self.current_state = CameraControlState::EnabledCursorUnlocked;

        Some(CameraStateChanged::new(previous, self.current_state))
    }

    /// 禁用相机
    pub fn disable(&mut self) -> Option<CameraStateChanged> {
        if !self.current_state.is_enabled() {
            return None; // 已经禁用
        }

        let previous = self.current_state;
        self.current_state = CameraControlState::Disabled;

        Some(CameraStateChanged::new(previous, self.current_state))
    }

    /// 切换光标锁定状态
    ///
    /// 如果相机已禁用，此操作无效
    pub fn toggle_cursor_lock(&mut self) -> Option<CameraStateChanged> {
        if !self.current_state.is_enabled() {
            return None; // 相机禁用时无法切换
        }

        let previous = self.current_state;
        self.current_state = match self.current_state {
            CameraControlState::EnabledCursorLocked => CameraControlState::EnabledCursorUnlocked,
            CameraControlState::EnabledCursorUnlocked => CameraControlState::EnabledCursorLocked,
            CameraControlState::Disabled => return None,
        };

        Some(CameraStateChanged::new(previous, self.current_state))
    }

    /// 锁定光标（仅在相机启用时有效）
    pub fn lock_cursor(&mut self) -> Option<CameraStateChanged> {
        if !self.current_state.is_enabled() {
            return None;
        }

        if self.current_state == CameraControlState::EnabledCursorLocked {
            return None; // 已经锁定
        }

        let previous = self.current_state;
        self.current_state = CameraControlState::EnabledCursorLocked;

        Some(CameraStateChanged::new(previous, self.current_state))
    }

    /// 解锁光标（仅在相机启用时有效）
    pub fn unlock_cursor(&mut self) -> Option<CameraStateChanged> {
        if !self.current_state.is_enabled() {
            return None;
        }

        if self.current_state == CameraControlState::EnabledCursorUnlocked {
            return None; // 已经解锁
        }

        let previous = self.current_state;
        self.current_state = CameraControlState::EnabledCursorUnlocked;

        Some(CameraStateChanged::new(previous, self.current_state))
    }

    /// 强制设置为指定状态
    pub fn set_state(&mut self, state: CameraControlState) -> Option<CameraStateChanged> {
        if self.current_state == state {
            return None;
        }

        let previous = self.current_state;
        self.current_state = state;

        Some(CameraStateChanged::new(previous, state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_states() {
        // 禁用状态
        let sm = CameraStateMachine::new(false, false);
        assert_eq!(sm.current_state(), CameraControlState::Disabled);
        assert!(!sm.is_enabled());
        assert!(!sm.is_fps_mode());

        // 启用且锁定
        let sm = CameraStateMachine::new(true, true);
        assert_eq!(sm.current_state(), CameraControlState::EnabledCursorLocked);
        assert!(sm.is_enabled());
        assert!(sm.is_fps_mode());

        // 启用但未锁定
        let sm = CameraStateMachine::new(true, false);
        assert_eq!(sm.current_state(), CameraControlState::EnabledCursorUnlocked);
        assert!(sm.is_enabled());
        assert!(!sm.is_fps_mode());
    }

    #[test]
    fn test_enable_disable_transitions() {
        let mut sm = CameraStateMachine::new(false, false);

        // 禁用 → 启用（未锁定）
        let change = sm.enable();
        assert!(change.is_some());
        assert_eq!(change.unwrap().current, CameraControlState::EnabledCursorUnlocked);

        // 重复启用应返回 None
        assert!(sm.enable().is_none());

        // 启用 → 禁用
        let change = sm.disable();
        assert!(change.is_some());
        assert_eq!(change.unwrap().current, CameraControlState::Disabled);

        // 重复禁用应返回 None
        assert!(sm.disable().is_none());
    }

    #[test]
    fn test_cursor_lock_transitions() {
        let mut sm = CameraStateMachine::new(true, false);

        // 解锁 → 锁定
        let change = sm.lock_cursor();
        assert!(change.is_some());
        assert!(sm.is_fps_mode());

        // 重复锁定应返回 None
        assert!(sm.lock_cursor().is_none());

        // 锁定 → 解锁
        let change = sm.unlock_cursor();
        assert!(change.is_some());
        assert!(!sm.is_fps_mode());

        // 重复解锁应返回 None
        assert!(sm.unlock_cursor().is_none());
    }

    #[test]
    fn test_toggle_cursor_lock() {
        let mut sm = CameraStateMachine::new(true, false);

        // 切换：解锁 → 锁定
        let change = sm.toggle_cursor_lock();
        assert!(change.is_some());
        assert!(sm.is_fps_mode());

        // 切换：锁定 → 解锁
        let change = sm.toggle_cursor_lock();
        assert!(change.is_some());
        assert!(!sm.is_fps_mode());

        // 禁用状态下切换应返回 None
        sm.disable();
        assert!(sm.toggle_cursor_lock().is_none());
    }

    #[test]
    fn test_disabled_camera_cursor_operations() {
        let mut sm = CameraStateMachine::new(false, false);

        // 禁用状态下所有光标操作应返回 None
        assert!(sm.lock_cursor().is_none());
        assert!(sm.unlock_cursor().is_none());
        assert!(sm.toggle_cursor_lock().is_none());
    }

    #[test]
    fn test_enable_with_cursor_locked() {
        let mut sm = CameraStateMachine::new(false, false);

        // 直接启用并锁定
        let change = sm.enable_with_cursor_locked();
        assert!(change.is_some());
        assert_eq!(change.unwrap().current, CameraControlState::EnabledCursorLocked);
        assert!(sm.is_fps_mode());
    }

    #[test]
    fn test_state_display() {
        assert_eq!(format!("{}", CameraControlState::Disabled), "Disabled");
        assert_eq!(
            format!("{}", CameraControlState::EnabledCursorLocked),
            "Enabled (FPS Mode)"
        );
        assert_eq!(
            format!("{}", CameraControlState::EnabledCursorUnlocked),
            "Enabled (Edit Mode)"
        );
    }
}
