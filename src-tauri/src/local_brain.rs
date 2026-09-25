use chrono::Local;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent { Greeting, Time, Memory, Reminder, Note, Unknown }

pub fn detect_intent(input: &str) -> Intent {
    let q = input.trim().to_lowercase();
    if q.is_empty() { return Intent::Unknown; }
    if ["hello","hi","hey","مرحبا","اهلا","أهلا","السلام عليكم"].iter().any(|x| q.contains(x)) { return Intent::Greeting; }
    if q.contains("time") || q.contains("الوقت") || q.contains("الساعة") || q.contains("كم الساعة") { return Intent::Time; }
    if q.contains("memory") || q.contains("remember") || q.contains("ذاكرة") || q.contains("تذكر") { return Intent::Memory; }
    if q.contains("remind") || q.contains("reminder") || q.contains("تذكير") || q.contains("ذكرني") { return Intent::Reminder; }
    if q.contains("note") || q.contains("ملاحظة") || q.contains("احفظ") || q.contains("سجل") { return Intent::Note; }
    Intent::Unknown
}

pub fn reply(input: &str, language: &str) -> String {
    match detect_intent(input) {
        Intent::Greeting => if language == "ar" { "مرحباً، أنا آدم. أنا متاح في الوضع المحلي.".into() } else { "Hello, I’m Adam. I’m available in local mode.".into() },
        Intent::Time => {
            let now = Local::now().format("%Y-%m-%d %H:%M").to_string();
            if language == "ar" { format!("الوقت المحلي الآن: {now}") } else { format!("The local time is {now}.") }
        },
        Intent::Memory => if language == "ar" { "يمكنني البحث في ذاكرتك المحلية وحفظ الملاحظات وتعديلها وحذفها.".into() } else { "I can search local memory and save, edit, or delete notes.".into() },
        Intent::Reminder => if language == "ar" { "يمكنني إنشاء التذكيرات المحلية وإكمالها.".into() } else { "I can create and complete local reminders.".into() },
        Intent::Note => if language == "ar" { "أستطيع حفظ هذه المعلومة كملاحظة محلية.".into() } else { "I can save that information as a local note.".into() },
        Intent::Unknown => if language == "ar" { "لا يوجد نموذج سحابي متاح حالياً. يمكنني مع ذلك استخدام الذاكرة والملاحظات والتذكيرات والصوت المحلي.".into() } else { "No cloud model is available right now. I can still use local memory, notes, reminders, and voice.".into() }
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_intent,reply,Intent};
    #[test] fn greeting_is_detected(){assert_eq!(detect_intent("hello"),Intent::Greeting);}
    #[test] fn time_is_detected(){assert_eq!(detect_intent("what time is it"),Intent::Time);}
    #[test] fn arabic_reminder_is_detected(){assert_eq!(detect_intent("ذكرني غداً"),Intent::Reminder);}
    #[test] fn note_is_detected(){assert_eq!(detect_intent("save this as a note"),Intent::Note);}
    #[test] fn unknown_has_local_fallback(){assert!(reply("something else","en").contains("No cloud"));}
}
