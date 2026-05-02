use crate::store::{TaskKind, TaskPriority, TaskStatus};

pub(crate) fn priority_emoji(priority: TaskPriority) -> &'static str {
    match priority {
        TaskPriority::High => "🔥",
        TaskPriority::Medium => "🌶️",
        TaskPriority::Low => "🧊",
    }
}

pub(crate) fn kind_emoji(kind: TaskKind) -> &'static str {
    match kind {
        TaskKind::Feature => "✨",
        TaskKind::Bug => "🐛",
        TaskKind::Chore => "🔧",
    }
}

pub(crate) fn status_label(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Todo => "Todo",
        TaskStatus::InProgress => "In Progress",
        TaskStatus::Done => "Done",
    }
}

#[cfg(test)]
mod tests {
    use super::{kind_emoji, priority_emoji, status_label};
    use crate::store::{TaskKind, TaskPriority, TaskStatus};

    #[test]
    fn priority_emoji_maps_all_priorities() {
        assert_eq!(priority_emoji(TaskPriority::High), "🔥");
        assert_eq!(priority_emoji(TaskPriority::Medium), "🌶️");
        assert_eq!(priority_emoji(TaskPriority::Low), "🧊");
    }

    #[test]
    fn kind_emoji_maps_all_kinds() {
        assert_eq!(kind_emoji(TaskKind::Feature), "✨");
        assert_eq!(kind_emoji(TaskKind::Bug), "🐛");
        assert_eq!(kind_emoji(TaskKind::Chore), "🔧");
    }

    #[test]
    fn status_label_maps_all_statuses() {
        assert_eq!(status_label(&TaskStatus::Todo), "Todo");
        assert_eq!(status_label(&TaskStatus::InProgress), "In Progress");
        assert_eq!(status_label(&TaskStatus::Done), "Done");
    }
}
