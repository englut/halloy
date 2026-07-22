use iced::Subscription;

#[cfg(target_os = "macos")]
mod macos;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Suspending,
    Resumed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Awake,
    Suspended,
}

impl State {
    pub fn suppresses_connection_events(self) -> bool {
        !matches!(self, Self::Awake)
    }
}

#[cfg(target_os = "macos")]
pub fn events() -> Subscription<Event> {
    macos::events()
}

#[cfg(not(target_os = "macos"))]
pub fn events() -> Subscription<Event> {
    Subscription::none()
}
