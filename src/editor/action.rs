use super::{main_editor::InsertModeTextAddInfo, mode::Mode};


#[derive(Debug)]
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
    UndoInsertModeTextAdd(InsertModeTextAddInfo), 
    InsertLineBelowCursor,
    GoToEndOfBuffer,
    PageUp,
    PageDown,
    DeleteFullLine,
    EnterWaitingMode(char),
    EnterMode(Mode),
    Undo,
    CenterLineToViewport,
    GoToStartOfBuffer,
    Backspace,
}
