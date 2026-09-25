use chrono::{Duration, Local};
use crate::{memory, reminders};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent { Greeting, Time, Memory, Reminder, Note, Unknown }

pub fn detect_intent(input: &str) -> Intent {
    let q = input.trim().to_lowercase();
    if q.is_empty() { return Intent::Unknown; }

    if q.contains("remind") || q.contains("reminder") || q.contains("تذكير") || q.contains("ذكرني") {
        return Intent::Reminder;
    }
    if q.contains("note") || q.contains("ملاحظة") || q.contains("احفظ") || q.contains("سجل") || q.contains("save this") {
        return Intent::Note;
    }
    if q.contains("memory") || q.contains("remember") || q.contains("ذاكرة") || q.contains("تذكر") {
        return Intent::Memory;
    }
    if q.contains("time") || q.contains("الوقت") || q.contains("الساعة") || q.contains("كم الساعة") {
        return Intent::Time;
    }
    if ["hello","hi","hey","مرحبا","اهلا","أهلا","السلام عليكم"].iter().any(|x| q.contains(x)) {
        return Intent::Greeting;
    }
    Intent::Unknown
}

fn is_ar(language: &str) -> bool { language.eq_ignore_ascii_case("ar") }

fn extract_after_any<'a>(input: &'a str, markers: &[&str]) -> &'a str {
    let lower = input.to_lowercase();
    for marker in markers {
        if let Some(pos) = lower.find(&marker.to_lowercase()) {
            return input[pos + marker.len()..].trim().trim_matches([':', '-', ' ']);
        }
    }
    input.trim()
}

fn reminder_due(input: &str) -> chrono::DateTime<Local> {
    let q = input.to_lowercase();
    let now = Local::now();
    if q.contains("today") || q.contains("اليوم") { return now + Duration::hours(1); }
    if q.contains("tomorrow") || q.contains("غدا") || q.contains("غداً") { return now + Duration::days(1); }
    if q.contains("next week") || q.contains("الأسبوع القادم") || q.contains("الاسبوع القادم") { return now + Duration::days(7); }
    now + Duration::hours(1)
}

pub fn execute(path: &Path, input: &str, language: &str) -> Result<Option<String>, String> {
    match detect_intent(input) {
        Intent::Greeting => Ok(Some(if is_ar(language) { "مرحباً، أنا آدم. أنا متاح في الوضع المحلي.".into() } else { "Hello, I’m Adam. I’m available in local mode.".into() })),
        Intent::Time => {
            let now = Local::now().format("%Y-%m-%d %H:%M").to_string();
            Ok(Some(if is_ar(language) { format!("الوقت المحلي الآن: {now}") } else { format!("The local time is {now}.") }))
        }
        Intent::Note => {
            let content = extract_after_any(input, &["save this as a note", "save this", "note", "احفظ", "سجل", "ملاحظة"]);
            if content.is_empty() { return Ok(Some(if is_ar(language) { "ماذا تريد أن أحفظ؟".into() } else { "What would you like me to save?".into() })); }
            let id = memory::add(path, content, "note")?;
            Ok(Some(if is_ar(language) { format!("تم حفظ الملاحظة رقم {id}.") } else { format!("Saved the note as memory #{id}.") }))
        }
        Intent::Memory => {
            let q = extract_after_any(input, &["search memory", "find in memory", "memory", "remember", "ابحث في الذاكرة", "ذاكرة", "تذكر"]);
            if q.is_empty() || q == input.trim() {
                let rows = memory::list(path, 5)?;
                if rows.is_empty() { return Ok(Some(if is_ar(language) { "الذاكرة المحلية فارغة.".into() } else { "Your local memory is empty.".into() })); }
                let body = rows.into_iter().map(|(_,c,_,_)| format!("- {c}")).collect::<Vec<_>>().join("\n");
                return Ok(Some(if is_ar(language) { format!("آخر ما أتذكره:\n{body}") } else { format!("Here is what I remember:\n{body}") }));
            }
            let rows = memory::search(path, q, 5)?;
            if rows.is_empty() { return Ok(Some(if is_ar(language) { "لم أجد شيئاً مطابقاً في الذاكرة المحلية.".into() } else { "I found nothing matching that in local memory.".into() })); }
            let body = rows.into_iter().map(|(_,c,_,_)| format!("- {c}")).collect::<Vec<_>>().join("\n");
            Ok(Some(if is_ar(language) { format!("وجدت في الذاكرة:\n{body}") } else { format!("I found in memory:\n{body}") }))
        }
        Intent::Reminder => {
            let title = extract_after_any(input, &["remind me to", "remind me", "reminder", "تذكير", "ذكرني"]);
            if title.is_empty() { return Ok(Some(if is_ar(language) { "ماذا تريد أن أذكرك به؟".into() } else { "What should I remind you about?".into() })); }
            let due = reminder_due(input).to_rfc3339();
            let id = reminders::add(path, title, &due)?;
            Ok(Some(if is_ar(language) { format!("تم إنشاء التذكير رقم {id} لـ {}.", due) } else { format!("Reminder #{id} created for {}.", due) }))
        }
        Intent::Unknown => Ok(None),
    }
}

pub fn reply(input: &str, language: &str) -> String {
    match detect_intent(input) {
        Intent::Unknown => if is_ar(language) { "لم أفهم الطلب محلياً.".into() } else { "I did not understand that locally.".into() },
        _ => execute(Path::new(":memory:"), input, language).unwrap_or_else(|_| {
            if is_ar(language) { "تعذر تنفيذ الطلب محلياً.".into() } else { "The local request could not be completed.".into() }
        }).unwrap_or_else(|| if is_ar(language) { "لم أفهم الطلب محلياً.".into() } else { "I did not understand that locally.".into() }),
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_intent, reply, Intent};
    #[test] fn greeting_is_detected(){assert_eq!(detect_intent("hello"),Intent::Greeting);}
    #[test] fn time_is_detected(){assert_eq!(detect_intent("what time is it"),Intent::Time);}
    #[test] fn arabic_reminder_is_detected(){assert_eq!(detect_intent("ذكرني غداً"),Intent::Reminder);}
    #[test] fn note_is_detected(){assert_eq!(detect_intent("save this as a note"),Intent::Note);}
    #[test] fn unknown_is_not_handled_locally(){assert_eq!(detect_intent("explain quantum tunneling"),Intent::Unknown);}
}