pub enum ButtonPressed {
    PauseButton,
    EncoderButton,
}

pub enum DeviceState {
    Running,
    Paused,
    SelectingChannel,
    SettingDivisionState,
}

impl DeviceState {
    pub fn transition(self, button: ButtonPressed) -> DeviceState {
        match (self, button) {
            (DeviceState::Running, ButtonPressed::PauseButton) => DeviceState::Paused,
            (DeviceState::Running, ButtonPressed::EncoderButton) => DeviceState::SelectingChannel,

            (DeviceState::Paused, ButtonPressed::PauseButton) => DeviceState::Running,
            (DeviceState::Paused, ButtonPressed::EncoderButton) => DeviceState::SelectingChannel,

            (DeviceState::SelectingChannel, ButtonPressed::PauseButton) => DeviceState::Running,
            (DeviceState::SelectingChannel, ButtonPressed::EncoderButton) => {
                DeviceState::SettingDivisionState
            }

            (DeviceState::SettingDivisionState, ButtonPressed::PauseButton) => DeviceState::Running,
            (DeviceState::SettingDivisionState, ButtonPressed::EncoderButton) => {
                DeviceState::SelectingChannel
            }
        }
    }
}
