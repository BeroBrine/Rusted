use super::mode::Mode;

pub enum Action {
    Quit,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveToEndOfLine,
    MoveToBeginningOfLine,
    InsertCharCursorPos(char),
    DeleteCharCursorPos,
    PageUp,
    PageDown,
    DeleteFullLine,
    EnterWaitingMode(char),
    EnterMode(Mode),
    Undo,
}
