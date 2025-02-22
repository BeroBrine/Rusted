use super::{main_editor::InsertChanges, mode::Mode};


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
    UndoInsertChanges(InsertChanges), 
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
}
