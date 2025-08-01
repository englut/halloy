use std::time::Duration;

use data::config::{self, Config, sidebar};
use data::dashboard::{BufferAction, BufferFocusedAction};
use data::{Version, buffer, file_transfer, history};
use iced::widget::{
    Column, Row, Scrollable, Space, button, column, container, horizontal_rule,
    horizontal_space, pane_grid, row, scrollable, stack, text, vertical_rule,
    vertical_space,
};
use iced::{Alignment, Length, Task, padding};
use itertools::Either;
use tokio::time;

use super::{Focus, Panes, Server};
use crate::widget::{Element, Text, context_menu, double_pass};
use crate::{Theme, font, icon, theme, window};

const CONFIG_RELOAD_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub enum Message {
    New(buffer::Upstream),
    Popout(buffer::Upstream),
    Focus(window::Id, pane_grid::Pane),
    Replace(buffer::Upstream),
    Close(window::Id, pane_grid::Pane),
    Swap(window::Id, pane_grid::Pane),
    Leave(buffer::Upstream),
    ToggleInternalBuffer(buffer::Internal),
    ToggleCommandBar,
    ToggleThemeEditor,
    ReloadConfigFile,
    ConfigReloaded(Result<Config, config::Error>),
    OpenReleaseWebsite,
    OpenDocumentation,
    OpenConfigFile,
    ReloadComplete,
    MarkAsRead(buffer::Upstream),
    MarkServerAsRead(Server),
}

#[derive(Debug, Clone)]
pub enum Event {
    New(buffer::Upstream),
    Popout(buffer::Upstream),
    Focus(window::Id, pane_grid::Pane),
    Replace(buffer::Upstream),
    Close(window::Id, pane_grid::Pane),
    Swap(window::Id, pane_grid::Pane),
    Leave(buffer::Upstream),
    ToggleInternalBuffer(buffer::Internal),
    ToggleCommandBar,
    ToggleThemeEditor,
    OpenReleaseWebsite,
    OpenDocumentation,
    OpenConfigFile,
    ConfigReloaded(Result<Config, config::Error>),
    MarkAsRead(buffer::Upstream),
    MarkServerAsRead(Server),
}

#[derive(Clone)]
pub struct Sidebar {
    pub hidden: bool,
    reloading_config: bool,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            hidden: false,
            reloading_config: false,
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.hidden = !self.hidden;
    }

    pub fn update(
        &mut self,
        message: Message,
    ) -> (Task<Message>, Option<Event>) {
        match message {
            Message::New(source) => (Task::none(), Some(Event::New(source))),
            Message::Popout(source) => {
                (Task::none(), Some(Event::Popout(source)))
            }
            Message::Focus(window, pane) => {
                (Task::none(), Some(Event::Focus(window, pane)))
            }
            Message::Replace(source) => {
                (Task::none(), Some(Event::Replace(source)))
            }
            Message::Close(window, pane) => {
                (Task::none(), Some(Event::Close(window, pane)))
            }
            Message::Swap(window, pane) => {
                (Task::none(), Some(Event::Swap(window, pane)))
            }
            Message::Leave(buffer) => {
                (Task::none(), Some(Event::Leave(buffer)))
            }
            Message::ToggleInternalBuffer(buffer) => {
                (Task::none(), Some(Event::ToggleInternalBuffer(buffer)))
            }
            Message::ToggleCommandBar => {
                (Task::none(), Some(Event::ToggleCommandBar))
            }
            Message::ToggleThemeEditor => {
                (Task::none(), Some(Event::ToggleThemeEditor))
            }
            Message::ReloadConfigFile => {
                self.reloading_config = true;
                (Task::perform(Config::load(), Message::ConfigReloaded), None)
            }
            Message::ConfigReloaded(config) => (
                Task::perform(time::sleep(CONFIG_RELOAD_DELAY), |()| {
                    Message::ReloadComplete
                }),
                Some(Event::ConfigReloaded(config)),
            ),
            Message::OpenReleaseWebsite => {
                (Task::none(), Some(Event::OpenReleaseWebsite))
            }
            Message::ReloadComplete => {
                self.reloading_config = false;
                (Task::none(), None)
            }
            Message::OpenDocumentation => {
                (Task::none(), Some(Event::OpenDocumentation))
            }
            Message::MarkAsRead(buffer) => {
                (Task::none(), Some(Event::MarkAsRead(buffer)))
            }
            Message::MarkServerAsRead(server) => {
                (Task::none(), Some(Event::MarkServerAsRead(server)))
            }
            Message::OpenConfigFile => {
                (Task::none(), Some(Event::OpenConfigFile))
            }
        }
    }

    fn user_menu_button<'a>(
        &self,
        keyboard: &'a data::config::Keyboard,
        file_transfers: &'a file_transfer::Manager,
        version: &'a Version,
        theme: &'a Theme,
    ) -> Element<'a, Message> {
        let base = button(icon::menu()).padding(5).width(Length::Shrink);

        let menu = Menu::list();

        // we show notification dot if theres a new version, or if theres transfers.
        let show_notification_dot =
            version.is_old() || !file_transfers.is_empty();

        if menu.is_empty() {
            base.into()
        } else {
            stack![
                context_menu(
                    context_menu::MouseButton::Left,
                    context_menu::Anchor::Widget,
                    context_menu::ToggleBehavior::Close,
                    base,
                    menu,
                    move |menu, length| {
                        let context_button =
                            |title: Text<'a>,
                             keybind: Option<&data::shortcut::KeyBind>,
                             icon: Text<'a>,
                             message: Message| {
                                button(
                                    row![
                                        icon.width(Length::Fixed(12.0)),
                                        title,
                                        keybind.map(|kb| {
                                            text(format!("({kb})"))
                                            .shaping(text::Shaping::Advanced)
                                            .size(theme::TEXT_SIZE - 2.0)
                                            .style(theme::text::secondary)
                                            .font_maybe(
                                                theme::font_style::secondary(
                                                    theme,
                                                )
                                                .map(font::get),
                                            )
                                        }),
                                    ]
                                    .spacing(8)
                                    .align_y(iced::Alignment::Center),
                                )
                                .width(length)
                                .padding(5)
                                .on_press(message)
                                .into()
                            };

                        match menu {
                            Menu::RefreshConfig => context_button(
                                text("Reload config file"),
                                Some(&keyboard.reload_configuration),
                                icon::refresh(),
                                Message::ReloadConfigFile,
                            ),
                            Menu::CommandBar => context_button(
                                text("Command Bar"),
                                Some(&keyboard.command_bar),
                                icon::search(),
                                Message::ToggleCommandBar,
                            ),
                            Menu::FileTransfers => context_button(
                                text("File Transfers")
                                    .style(if file_transfers.is_empty() {
                                        theme::text::primary
                                    } else {
                                        theme::text::tertiary
                                    })
                                    .font_maybe(if file_transfers.is_empty() {
                                        theme::font_style::primary(theme)
                                            .map(font::get)
                                    } else {
                                        theme::font_style::tertiary(theme)
                                            .map(font::get)
                                    }),
                                Some(&keyboard.file_transfers),
                                icon::file_transfer().style(
                                    if file_transfers.is_empty() {
                                        theme::text::primary
                                    } else {
                                        theme::text::tertiary
                                    },
                                ),
                                Message::ToggleInternalBuffer(
                                    buffer::Internal::FileTransfers,
                                ),
                            ),
                            Menu::Highlights => context_button(
                                text("Highlights"),
                                Some(&keyboard.highlights),
                                icon::highlights(),
                                Message::ToggleInternalBuffer(
                                    buffer::Internal::Highlights,
                                ),
                            ),
                            Menu::Logs => context_button(
                                text("Logs"),
                                Some(&keyboard.logs),
                                icon::logs(),
                                Message::ToggleInternalBuffer(
                                    buffer::Internal::Logs,
                                ),
                            ),
                            Menu::ThemeEditor => context_button(
                                text("Theme Editor"),
                                Some(&keyboard.theme_editor),
                                icon::theme_editor(),
                                Message::ToggleThemeEditor,
                            ),
                            Menu::HorizontalRule => match length {
                                Length::Fill => container(horizontal_rule(1))
                                    .padding([0, 6])
                                    .into(),
                                _ => Space::new(length, 1).into(),
                            },
                            Menu::Version => match version.is_old() {
                                true => context_button(
                                    text("New version available")
                                        .style(theme::text::tertiary)
                                        .font_maybe(
                                            theme::font_style::tertiary(theme)
                                                .map(font::get),
                                        ),
                                    None,
                                    icon::megaphone()
                                        .style(theme::text::tertiary),
                                    Message::OpenReleaseWebsite,
                                ),
                                false => container(
                                    text(format!(
                                        "Halloy ({})",
                                        version.current
                                    ))
                                    .style(theme::text::secondary)
                                    .font_maybe(
                                        theme::font_style::secondary(theme)
                                            .map(font::get),
                                    ),
                                )
                                .padding(5)
                                .into(),
                            },
                            Menu::Documentation => context_button(
                                text("Documentation"),
                                None,
                                icon::documentation(),
                                Message::OpenDocumentation,
                            ),
                            Menu::OpenConfigFile => context_button(
                                text("Open config file"),
                                None,
                                icon::config(),
                                Message::OpenConfigFile,
                            ),
                        }
                    },
                ),
                if show_notification_dot {
                    Some(
                        container(
                            icon::dot().style(theme::text::tertiary).size(8),
                        )
                        .padding(padding::left(13).top(2)),
                    )
                } else {
                    None
                },
            ]
            .into()
        }
    }

    pub fn view<'a>(
        &'a self,
        clients: &data::client::Map,
        history: &'a history::Manager,
        panes: &'a Panes,
        focus: Focus,
        config: &'a Config,
        file_transfers: &'a file_transfer::Manager,
        version: &'a Version,
        theme: &'a Theme,
    ) -> Option<Element<'a, Message>> {
        if self.hidden {
            return None;
        }

        let content = |width| {
            let user_menu_button = config.sidebar.show_user_menu.then(|| {
                self.user_menu_button(
                    &config.keyboard,
                    file_transfers,
                    version,
                    theme,
                )
            });

            let mut buffers = vec![];
            let mut client_enumeration = 0;

            let servers = match config.sidebar.order_by {
                sidebar::OrderBy::Alpha => Either::Left(clients.servers()),
                sidebar::OrderBy::Config => Either::Right(
                    config.servers.keys().chain(
                        clients
                            .servers()
                            .filter(|key| !config.servers.contains(key)),
                    ),
                ),
            };

            for server in servers {
                let button = |buffer: buffer::Upstream,
                              connected: bool,
                              server_has_unread: bool,
                              has_unread: bool| {
                    upstream_buffer_button(
                        panes,
                        focus,
                        buffer,
                        connected,
                        config.actions.sidebar.buffer,
                        config.actions.sidebar.focused_buffer,
                        config.sidebar.position,
                        config.sidebar.unread_indicator,
                        server_has_unread,
                        has_unread,
                        width,
                        theme,
                    )
                };

                if let Some(state) = clients.state(server) {
                    client_enumeration += 1;

                    match state {
                        data::client::State::Disconnected => {
                            // Disconnected server.
                            buffers.push(button(
                                buffer::Upstream::Server(server.clone()),
                                false,
                                history.server_has_unread(server.clone()),
                                history.has_unread(&history::Kind::Server(
                                    server.clone(),
                                )),
                            ));
                        }
                        data::client::State::Ready(connection) => {
                            // Connected server.
                            buffers.push(button(
                                buffer::Upstream::Server(server.clone()),
                                true,
                                history.server_has_unread(server.clone()),
                                history.has_unread(&history::Kind::Server(
                                    server.clone(),
                                )),
                            ));

                            // Channels from the connected server.
                            for channel in connection.channels() {
                                buffers.push(button(
                                    buffer::Upstream::Channel(
                                        server.clone(),
                                        channel.clone(),
                                    ),
                                    true,
                                    history.server_has_unread(server.clone()),
                                    history.has_unread(
                                        &history::Kind::Channel(
                                            server.clone(),
                                            channel.clone(),
                                        ),
                                    ),
                                ));
                            }

                            // Queries from the connected server.
                            let queries = history.get_unique_queries(server);
                            for query in queries {
                                let query = clients
                                    .resolve_query(server, query)
                                    .unwrap_or(query);

                                buffers.push(button(
                                    buffer::Upstream::Query(
                                        server.clone(),
                                        query.clone(),
                                    ),
                                    true,
                                    history.server_has_unread(server.clone()),
                                    history.has_unread(&history::Kind::Query(
                                        server.clone(),
                                        query.clone(),
                                    )),
                                ));
                            }

                            // Separator between servers.
                            if config.sidebar.position.is_horizontal() {
                                if client_enumeration < clients.len() {
                                    buffers.push(
                                        container(vertical_rule(1))
                                            .padding(padding::top(6))
                                            .height(20)
                                            .width(12)
                                            .align_x(Alignment::Center)
                                            .into(),
                                    );
                                }
                            } else {
                                buffers
                                    .push(vertical_space().height(12).into());
                            }
                        }
                    }
                }
            }

            match config.sidebar.position {
                sidebar::Position::Left | sidebar::Position::Right => {
                    // Add buffers to a column.
                    let buffers = column![
                        Scrollable::new(
                            Column::with_children(buffers).spacing(1)
                        )
                        .direction(
                            scrollable::Direction::Vertical(
                                scrollable::Scrollbar::default()
                                    .width(config.sidebar.scrollbar.width)
                                    .scroller_width(
                                        config.sidebar.scrollbar.scroller_width
                                    )
                            )
                        )
                    ];

                    // Wrap buffers in a column with user_menu_button
                    let content = column![
                        container(buffers).height(Length::Fill),
                        user_menu_button,
                    ];

                    container(content)
                }
                sidebar::Position::Top | sidebar::Position::Bottom => {
                    // Add buffers to a row.
                    let buffers = row![
                        Scrollable::new(Row::with_children(buffers).spacing(2))
                            .direction(scrollable::Direction::Horizontal(
                                scrollable::Scrollbar::default()
                                    .width(config.sidebar.scrollbar.width)
                                    .scroller_width(
                                        config.sidebar.scrollbar.scroller_width
                                    )
                            ))
                    ];

                    // Wrap buffers in a row with user_menu_button
                    let content = row![
                        container(buffers).width(Length::Fill),
                        user_menu_button,
                    ]
                    .align_y(Alignment::Center);

                    container(content)
                }
            }
        };

        let padding = match config.sidebar.position {
            sidebar::Position::Left => padding::top(8).bottom(6).left(6),
            sidebar::Position::Right => padding::top(8).bottom(6).right(6),
            sidebar::Position::Top => padding::top(8).left(6).right(6),
            sidebar::Position::Bottom => padding::bottom(8).left(6).right(6),
        };

        let content = if config.sidebar.position.is_horizontal() {
            container(
                content(Length::Shrink).width(Length::Fill).padding(padding),
            )
            .into()
        } else {
            let first_pass = content(Length::Shrink);
            let second_pass = content(Length::Fill);

            container(double_pass(first_pass, second_pass))
                .max_width(
                    config.sidebar.max_width.map_or(f32::INFINITY, f32::from),
                )
                .width(Length::Shrink)
                .padding(padding)
                .into()
        };

        Some(content)
    }
}

#[derive(Debug, Clone, Copy)]
enum Menu {
    RefreshConfig,
    CommandBar,
    ThemeEditor,
    Highlights,
    Logs,
    FileTransfers,
    Version,
    HorizontalRule,
    Documentation,
    OpenConfigFile,
}

impl Menu {
    fn list() -> Vec<Self> {
        vec![
            Menu::Version,
            Menu::HorizontalRule,
            Menu::CommandBar,
            Menu::Documentation,
            Menu::FileTransfers,
            Menu::Highlights,
            Menu::Logs,
            Menu::OpenConfigFile,
            Menu::RefreshConfig,
            Menu::ThemeEditor,
        ]
    }
}

#[derive(Debug, Clone, Copy)]
enum Entry {
    MarkServerAsRead,
    MarkAsRead,
    NewPane,
    Popout,
    Replace,
    Close(window::Id, pane_grid::Pane),
    Swap(window::Id, pane_grid::Pane),
    Leave,
}

impl Entry {
    fn list(
        buffer: &buffer::Upstream,
        num_panes: usize,
        open: Option<(window::Id, pane_grid::Pane)>,
        focus: Focus,
    ) -> Vec<Self> {
        [
            match buffer {
                buffer::Upstream::Server(_) => {
                    vec![Entry::MarkServerAsRead]
                }
                buffer::Upstream::Channel(_, _) => vec![],
                buffer::Upstream::Query(_, _) => vec![],
            },
            match open {
                None => vec![
                    Entry::MarkAsRead,
                    Entry::NewPane,
                    Entry::Popout,
                    Entry::Replace,
                    Entry::Leave,
                ],
                Some((window, pane)) => (num_panes > 1)
                    .then_some(Entry::Close(window, pane))
                    .into_iter()
                    .chain(
                        (Focus { window, pane } != focus)
                            .then_some(Entry::Swap(window, pane)),
                    )
                    .chain(Some(Entry::Leave))
                    .collect(),
            },
        ]
        .concat()
    }
}

fn upstream_buffer_button<'a>(
    panes: &'a Panes,
    focus: Focus,
    buffer: buffer::Upstream,
    connected: bool,
    buffer_action: BufferAction,
    focused_buffer_action: Option<BufferFocusedAction>,
    position: sidebar::Position,
    unread_indicator: sidebar::UnreadIndicator,
    server_has_unread: bool,
    has_unread: bool,
    width: Length,
    theme: &'a Theme,
) -> Element<'a, Message> {
    let open = panes.iter().find_map(|(window_id, pane, state)| {
        (state.buffer.upstream() == Some(&buffer)).then_some((window_id, pane))
    });
    let is_focused = panes.iter().find_map(|(window_id, pane, state)| {
        (Focus {
            window: window_id,
            pane,
        } == focus
            && state.buffer.upstream() == Some(&buffer))
        .then_some((window_id, pane))
    });

    let show_unread_indicator =
        has_unread && matches!(unread_indicator, sidebar::UnreadIndicator::Dot);
    let show_title_indicator = has_unread
        && matches!(unread_indicator, sidebar::UnreadIndicator::Title);

    let unread_dot_indicator_spacing =
        horizontal_space().width(match position.is_horizontal() {
            true => {
                if show_unread_indicator {
                    5
                } else {
                    0
                }
            }
            false => {
                if show_unread_indicator {
                    11
                } else {
                    16
                }
            }
        });
    let buffer_title_style = if show_title_indicator {
        theme::text::unread_indicator
    } else {
        theme::text::primary
    };
    let buffer_title_font = theme::font_style::primary(theme).map(font::get);

    let row = match &buffer {
        buffer::Upstream::Server(server) => row![
            icon::connected().style(if connected {
                if show_unread_indicator {
                    theme::text::unread_indicator
                } else {
                    theme::text::primary
                }
            } else {
                theme::text::error
            }),
            text(server.to_string())
                .style(buffer_title_style)
                .font_maybe(buffer_title_font)
                .shaping(text::Shaping::Advanced)
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        buffer::Upstream::Channel(_, channel) => row![
            horizontal_space().width(3),
            show_unread_indicator.then_some(
                icon::dot().size(6).style(theme::text::unread_indicator),
            ),
            unread_dot_indicator_spacing,
            text(channel.to_string())
                .style(buffer_title_style)
                .font_maybe(buffer_title_font)
                .shaping(text::Shaping::Advanced),
            horizontal_space().width(3),
        ]
        .align_y(iced::Alignment::Center),
        buffer::Upstream::Query(_, query) => row![
            horizontal_space().width(3),
            show_unread_indicator.then_some(
                icon::dot().size(6).style(theme::text::unread_indicator),
            ),
            unread_dot_indicator_spacing,
            text(query.to_string())
                .style(buffer_title_style)
                .font_maybe(buffer_title_font)
                .shaping(text::Shaping::Advanced),
            horizontal_space().width(3),
        ]
        .align_y(iced::Alignment::Center),
    };

    let base = button(row.width(width))
        .padding(5)
        .style(move |theme, status| {
            theme::button::sidebar_buffer(
                theme,
                status,
                is_focused.is_some(),
                open.is_some(),
            )
        })
        .on_press({
            match is_focused {
                Some((window, pane)) => {
                    if let Some(focus_action) = focused_buffer_action {
                        match focus_action {
                            BufferFocusedAction::ClosePane => {
                                Message::Close(window, pane)
                            }
                        }
                    } else {
                        // Re-focus pane on press instead of disabling the button in order
                        // to have hover status of the button for styling
                        Message::Focus(window, pane)
                    }
                }
                None => {
                    if let Some((window, pane)) = open {
                        Message::Focus(window, pane)
                    } else {
                        match buffer_action {
                            BufferAction::NewPane => {
                                Message::New(buffer.clone())
                            }
                            BufferAction::ReplacePane => {
                                Message::Replace(buffer.clone())
                            }
                            BufferAction::NewWindow => {
                                Message::Popout(buffer.clone())
                            }
                        }
                    }
                }
            }
        });

    let entries = Entry::list(&buffer, panes.len(), open, focus);

    if entries.is_empty() || !connected {
        base.into()
    } else {
        context_menu(
            context_menu::MouseButton::default(),
            context_menu::Anchor::Cursor,
            context_menu::ToggleBehavior::KeepOpen,
            base,
            entries,
            move |entry, length| {
                let (content, message) = match entry {
                    Entry::MarkServerAsRead => (
                        "Mark entire server as read",
                        if server_has_unread {
                            Some(Message::MarkServerAsRead(
                                buffer.server().clone(),
                            ))
                        } else {
                            None
                        },
                    ),
                    Entry::MarkAsRead => (
                        if matches!(&buffer, buffer::Upstream::Server(_)) {
                            "Mark server buffer as read"
                        } else {
                            "Mark as read"
                        },
                        if has_unread {
                            Some(Message::MarkAsRead(buffer.clone()))
                        } else {
                            None
                        },
                    ),
                    Entry::NewPane => {
                        ("Open in new pane", Some(Message::New(buffer.clone())))
                    }
                    Entry::Popout => (
                        "Open in new window",
                        Some(Message::Popout(buffer.clone())),
                    ),
                    Entry::Replace => (
                        "Replace current pane",
                        Some(Message::Replace(buffer.clone())),
                    ),
                    Entry::Close(window, pane) => {
                        ("Close pane", Some(Message::Close(window, pane)))
                    }
                    Entry::Swap(window, pane) => (
                        "Swap with current pane",
                        Some(Message::Swap(window, pane)),
                    ),
                    Entry::Leave => (
                        match &buffer {
                            buffer::Upstream::Server(_) => "Leave server",
                            buffer::Upstream::Channel(_, _) => "Leave channel",
                            buffer::Upstream::Query(_, _) => "Close query",
                        },
                        Some(Message::Leave(buffer.clone())),
                    ),
                };

                button(text(content))
                    .width(length)
                    .padding(5)
                    .style(|theme, status| {
                        theme::button::primary(theme, status, false)
                    })
                    .on_press_maybe(message)
                    .into()
            },
        )
        .into()
    }
}
