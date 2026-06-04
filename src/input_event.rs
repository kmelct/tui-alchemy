#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    FocusGained,
    FocusLost,
    Paste(()),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
    Esc,
    Enter,
    Backspace,
    Char(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyModifiers(u8);

impl KeyModifiers {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(1 << 0);
    pub const CONTROL: Self = Self(1 << 1);
    pub const ALT: Self = Self(1 << 2);
    pub const SUPER: Self = Self(1 << 3);

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventKind {
    Down(MouseButton),
    Up(MouseButton),
    Drag(MouseButton),
    ScrollDown,
    ScrollUp,
    Moved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub column: u16,
    pub row: u16,
    pub modifiers: KeyModifiers,
}

#[cfg(not(target_arch = "wasm32"))]
impl Event {
    pub fn from_crossterm(event: crossterm::event::Event) -> Option<Self> {
        use crossterm::event::{KeyCode as CtKeyCode, MouseButton as CtMouseButton};

        match event {
            crossterm::event::Event::Key(key) => Some(Self::Key(KeyEvent::new(
                match key.code {
                    CtKeyCode::Up => KeyCode::Up,
                    CtKeyCode::Down => KeyCode::Down,
                    CtKeyCode::Left => KeyCode::Left,
                    CtKeyCode::Right => KeyCode::Right,
                    CtKeyCode::PageUp => KeyCode::PageUp,
                    CtKeyCode::PageDown => KeyCode::PageDown,
                    CtKeyCode::Home => KeyCode::Home,
                    CtKeyCode::End => KeyCode::End,
                    CtKeyCode::Esc => KeyCode::Esc,
                    CtKeyCode::Enter => KeyCode::Enter,
                    CtKeyCode::Backspace => KeyCode::Backspace,
                    CtKeyCode::Char(ch) => KeyCode::Char(ch),
                    _ => return None,
                },
                KeyModifiers::from_crossterm(key.modifiers),
            ))),
            crossterm::event::Event::Mouse(mouse) => Some(Self::Mouse(MouseEvent {
                kind: match mouse.kind {
                    crossterm::event::MouseEventKind::Down(CtMouseButton::Left) => {
                        MouseEventKind::Down(MouseButton::Left)
                    }
                    crossterm::event::MouseEventKind::Down(CtMouseButton::Right) => {
                        MouseEventKind::Down(MouseButton::Right)
                    }
                    crossterm::event::MouseEventKind::Down(CtMouseButton::Middle) => {
                        MouseEventKind::Down(MouseButton::Middle)
                    }
                    crossterm::event::MouseEventKind::Up(CtMouseButton::Left) => {
                        MouseEventKind::Up(MouseButton::Left)
                    }
                    crossterm::event::MouseEventKind::Up(CtMouseButton::Right) => {
                        MouseEventKind::Up(MouseButton::Right)
                    }
                    crossterm::event::MouseEventKind::Up(CtMouseButton::Middle) => {
                        MouseEventKind::Up(MouseButton::Middle)
                    }
                    crossterm::event::MouseEventKind::Drag(CtMouseButton::Left) => {
                        MouseEventKind::Drag(MouseButton::Left)
                    }
                    crossterm::event::MouseEventKind::Drag(CtMouseButton::Right) => {
                        MouseEventKind::Drag(MouseButton::Right)
                    }
                    crossterm::event::MouseEventKind::Drag(CtMouseButton::Middle) => {
                        MouseEventKind::Drag(MouseButton::Middle)
                    }
                    crossterm::event::MouseEventKind::ScrollDown => MouseEventKind::ScrollDown,
                    crossterm::event::MouseEventKind::ScrollUp => MouseEventKind::ScrollUp,
                    crossterm::event::MouseEventKind::Moved => MouseEventKind::Moved,
                    _ => return None,
                },
                column: mouse.column,
                row: mouse.row,
                modifiers: KeyModifiers::from_crossterm(mouse.modifiers),
            })),
            crossterm::event::Event::Resize(width, height) => Some(Self::Resize(width, height)),
            crossterm::event::Event::FocusGained => Some(Self::FocusGained),
            crossterm::event::Event::FocusLost => Some(Self::FocusLost),
            crossterm::event::Event::Paste(_) => Some(Self::Paste(())),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl KeyModifiers {
    pub const fn from_crossterm(modifiers: crossterm::event::KeyModifiers) -> Self {
        let mut out = Self::NONE;
        if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
            out = out.union(Self::SHIFT);
        }
        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
            out = out.union(Self::CONTROL);
        }
        if modifiers.contains(crossterm::event::KeyModifiers::ALT) {
            out = out.union(Self::ALT);
        }
        if modifiers.contains(crossterm::event::KeyModifiers::SUPER) {
            out = out.union(Self::SUPER);
        }
        out
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod crossterm_tests {
    use super::*;
    use crossterm::event as ct;

    #[test]
    fn crossterm_key_events_keep_supported_codes_and_modifiers() {
        let event = Event::from_crossterm(ct::Event::Key(ct::KeyEvent::new(
            ct::KeyCode::Char('x'),
            ct::KeyModifiers::SHIFT | ct::KeyModifiers::CONTROL | ct::KeyModifiers::ALT,
        )));

        assert_eq!(
            event,
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('x'),
                KeyModifiers::SHIFT
                    .union(KeyModifiers::CONTROL)
                    .union(KeyModifiers::ALT),
            ))),
        );
    }

    #[test]
    fn crossterm_mouse_events_keep_position_button_and_modifiers() {
        let event = Event::from_crossterm(ct::Event::Mouse(ct::MouseEvent {
            kind: ct::MouseEventKind::Drag(ct::MouseButton::Right),
            column: 12,
            row: 7,
            modifiers: ct::KeyModifiers::SUPER,
        }));

        assert_eq!(
            event,
            Some(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Right),
                column: 12,
                row: 7,
                modifiers: KeyModifiers::SUPER,
            })),
        );
    }

    #[test]
    fn crossterm_non_game_events_are_filtered_or_normalized() {
        assert_eq!(
            Event::from_crossterm(ct::Event::Key(ct::KeyEvent::new(
                ct::KeyCode::F(1),
                ct::KeyModifiers::NONE,
            ))),
            None,
        );
        assert_eq!(
            Event::from_crossterm(ct::Event::Resize(80, 24)),
            Some(Event::Resize(80, 24))
        );
        assert_eq!(
            Event::from_crossterm(ct::Event::FocusGained),
            Some(Event::FocusGained)
        );
        assert_eq!(
            Event::from_crossterm(ct::Event::FocusLost),
            Some(Event::FocusLost)
        );
        assert_eq!(
            Event::from_crossterm(ct::Event::Paste("ignored".to_owned())),
            Some(Event::Paste(()))
        );
    }
}
