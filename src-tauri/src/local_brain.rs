use crate::{calendar, memory, reminders};
use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    Greeting,
    Time,
    Memory,
    Reminder,
    Note,
    Calendar,
    Unknown,
}

pub fn detect_intent(input: &str) -> Intent {
    let q = normalize(input);
    if q.is_empty() {
        return Intent::Unknown;
    }
    if q.contains("remind") || q.contains("reminder") || q.contains("تذكير") || q.contains("ذكرني")
    {
        return Intent::Reminder;
    }
    if q.contains("calendar")
        || q.contains("schedule")
        || q.contains("appointment")
        || q.contains("meeting")
        || q.contains("event")
        || q.contains("موعد")
        || q.contains("تقويم")
        || q.contains("جدول")
    {
        return Intent::Calendar;
    }
    if q.contains("note")
        || q.contains("write down")
        || q.contains("save a note")
        || q.contains("ملاحظة")
        || q.contains("احفظ")
        || q.contains("سجل")
        || q.contains("save this")
    {
        return Intent::Note;
    }
    if q.contains("memory")
        || q.contains("remember")
        || q.contains("what did i save")
        || q.contains("what do you remember")
        || q.contains("ذاكرة")
        || q.contains("تذكر")
    {
        return Intent::Memory;
    }
    if q.contains("time")
        || q.contains("what time")
        || q.contains("current time")
        || q.contains("الوقت")
        || q.contains("الساعة")
        || q.contains("كم الساعة")
    {
        return Intent::Time;
    }
    let greetings = [
        "hello",
        "hi",
        "hey",
        "good morning",
        "good evening",
        "مرحبا",
        "اهلا",
        "السلام عليكم",
    ];
    if greetings.iter().any(|x| q.contains(x))
    {
        return Intent::Greeting;
    }
    Intent::Unknown
}
fn normalize(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .replace("أ", "ا")
        .replace("إ", "ا")
        .replace("آ", "ا")
        .replace("ة", "ه")
        .replace("ى", "ي")
}
fn is_ar(language: &str) -> bool {
    language.eq_ignore_ascii_case("ar")
}
fn extract_after_any<'a>(input: &'a str, markers: &[&str]) -> &'a str {
    let trimmed = input.trim();
    for marker in markers {
        let lower = trimmed.to_lowercase();
        let ml = marker.to_lowercase();
        if let Some(pos) = lower.find(&ml) {
            let char_pos = lower[..pos].chars().count();
            let mut iter = trimmed.char_indices();
            let byte_pos = iter.nth(char_pos).map(|(i, _)| i).unwrap_or(trimmed.len());
            let marker_end = trimmed[byte_pos..]
                .char_indices()
                .nth(marker.chars().count())
                .map(|(i, _)| i)
                .unwrap_or(trimmed[byte_pos..].len());
            return trimmed[byte_pos + marker_end..]
                .trim()
                .trim_matches([':', '-', ' ']);
        }
    }
    trimmed
}
fn parse_due(input: &str) -> DateTime<Local> {
    let q = input.to_lowercase();
    let now = Local::now();
    if q.contains("in 10 minutes") {
        return now + Duration::minutes(10);
    }
    if q.contains("in 30 minutes") {
        return now + Duration::minutes(30);
    }
    if q.contains("in an hour") || q.contains("in 1 hour") || q.contains("بعد ساعه") {
        return now + Duration::hours(1);
    }
    if q.contains("tomorrow") || q.contains("غدا") || q.contains("غداً") {
        return now + Duration::days(1);
    }
    if q.contains("next week") || q.contains("الأسبوع القادم") || q.contains("الاسبوع القادم")
    {
        return now + Duration::days(7);
    }
    if q.contains("today") || q.contains("اليوم") {
        return parse_clock(&q, now.date_naive()).unwrap_or(now + Duration::hours(1));
    }
    if let Some(dt) = parse_clock(&q, now.date_naive()) {
        return if dt > now { dt } else { dt + Duration::days(1) };
    }
    now + Duration::hours(1)
}
fn parse_clock(q: &str, date: NaiveDate) -> Option<DateTime<Local>> {
    let words = q.split_whitespace().collect::<Vec<_>>();
    for (i, w) in words.iter().enumerate() {
        let s = w.trim_matches(|c: char| !c.is_ascii_digit() && c != ':');
        if s.contains(':') || s.chars().all(|c| c.is_ascii_digit()) {
            if !s.is_empty() && s.len() <= 5 {
                let mut parts = s.split(':');
                let h = parts.next()?.parse::<u32>().ok()?;
                let m = parts.next().unwrap_or("0").parse::<u32>().ok()?;
                if h < 24 && m < 60 {
                    let mut hour = h;
                    let mer = words.get(i + 1).copied().unwrap_or("");
                    if mer.contains("pm") || mer.contains("مساء") {
                        if hour < 12 {
                            hour += 12
                        }
                    } else if (mer.contains("am") || mer.contains("صباح")) && hour == 12 {
                        hour = 0
                    }
                    return Local
                        .from_local_datetime(&NaiveDateTime::new(
                            date,
                            NaiveTime::from_hms_opt(hour, m, 0)?,
                        ))
                        .single();
                }
            }
        }
    }
    None
}
pub fn execute(path: &Path, input: &str, language: &str) -> Result<Option<String>, String> {
    match detect_intent(input) {
        Intent::Greeting => Ok(Some(if is_ar(language) {
            "مرحباً، أنا آدم. أعمل محلياً عند توفر المهمة.".into()
        } else {
            "Hello, I’m Adam. I can execute supported tasks locally.".into()
        })),
        Intent::Time => {
            let now = Local::now().format("%Y-%m-%d %H:%M").to_string();
            Ok(Some(if is_ar(language) {
                format!("الوقت المحلي الآن: {now}")
            } else {
                format!("The local time is {now}.")
            }))
        }
        Intent::Note => {
            let content = extract_after_any(
                input,
                &[
                    "save this as a note",
                    "save this",
                    "note",
                    "احفظ",
                    "سجل",
                    "ملاحظة",
                ],
            );
            if content.is_empty() {
                return Ok(Some(if is_ar(language) {
                    "ماذا تريد أن أحفظ؟".into()
                } else {
                    "What would you like me to save?".into()
                }));
            }
            let id = memory::add(path, content, "note")?;
            Ok(Some(if is_ar(language) {
                format!("تم حفظ الملاحظة رقم {id}.")
            } else {
                format!("Saved note #{id}.")
            }))
        }
        Intent::Memory => {
            let q = extract_after_any(
                input,
                &[
                    "search memory",
                    "find in memory",
                    "memory",
                    "remember",
                    "ابحث في الذاكرة",
                    "ذاكرة",
                    "تذكر",
                ],
            );
            let rows = if q.is_empty() || q == input.trim() {
                memory::list(path, 5)?
            } else {
                memory::search(path, q, 5)?
            };
            if rows.is_empty() {
                return Ok(Some(if is_ar(language) {
                    "الذاكرة المحلية فارغة أو لا يوجد تطابق.".into()
                } else {
                    "No matching local memories were found.".into()
                }));
            }
            let body = rows
                .into_iter()
                .map(|(_, c, _, _)| format!("- {c}"))
                .collect::<Vec<_>>()
                .join(
                    "
",
                );
            Ok(Some(if is_ar(language) {
                format!(
                    "وجدت في الذاكرة:
{body}"
                )
            } else {
                format!(
                    "Here is what I found in local memory:
{body}"
                )
            }))
        }
        Intent::Reminder => {
            let title = extract_after_any(
                input,
                &["remind me to", "remind me", "reminder", "تذكير", "ذكرني"],
            );
            if title.is_empty() {
                return Ok(Some(if is_ar(language) {
                    "ماذا تريد أن أذكرك به؟".into()
                } else {
                    "What should I remind you about?".into()
                }));
            }
            let due = parse_due(input).to_rfc3339();
            let id = reminders::add(path, title, &due)?;
            Ok(Some(if is_ar(language) {
                format!("تم إنشاء التذكير رقم {id} لـ {due}.")
            } else {
                format!("Reminder #{id} created for {due}.")
            }))
        }
        Intent::Calendar => {
            let title = extract_after_any(
                input,
                &[
                    "add event",
                    "schedule",
                    "event",
                    "موعد",
                    "أضف موعد",
                    "اضف موعد",
                ],
            );
            if title.is_empty() || title == input.trim() {
                return Ok(Some(if is_ar(language) {
                    "ما اسم الموعد؟ مثال: أضف موعد طبيب غداً الساعة 10".into()
                } else {
                    "What is the event title? Example: schedule dentist tomorrow at 10.".into()
                }));
            }
            let due = parse_due(input).to_rfc3339();
            let id = calendar::add(path, title, &due)?;
            Ok(Some(if is_ar(language) {
                format!("تمت إضافة الموعد رقم {id} في {due}.")
            } else {
                format!("Calendar event #{id} added for {due}.")
            }))
        }
        Intent::Unknown => Ok(None),
    }
}
pub fn reply(input: &str, language: &str) -> String {
    match detect_intent(input) {
        Intent::Unknown => {
            if is_ar(language) {
                "لم أفهم الطلب محلياً.".into()
            } else {
                "I did not understand that locally.".into()
            }
        }
        _ => {
            if is_ar(language) {
                "تم تنفيذ الطلب محلياً.".into()
            } else {
                "The local request was handled.".into()
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::{detect_intent, Intent};
    #[test]
    fn greeting() {
        assert_eq!(detect_intent("hello"), Intent::Greeting)
    }
    #[test]
    fn time() {
        assert_eq!(detect_intent("what time is it"), Intent::Time)
    }
    #[test]
    fn arabic_reminder() {
        assert_eq!(detect_intent("ذكرني غداً"), Intent::Reminder)
    }
    #[test]
    fn note() {
        assert_eq!(detect_intent("save this as a note"), Intent::Note)
    }
    #[test]
    fn calendar() {
        assert_eq!(detect_intent("schedule dentist tomorrow"), Intent::Calendar)
    }
    #[test]
    fn relative_time_is_reminder() {
        assert_eq!(detect_intent("remind me in 30 minutes"), Intent::Reminder)
    }
    #[test]
    fn appointment_is_calendar() {
        let intent = detect_intent("add an appointment tomorrow");
        assert_eq!(intent, Intent::Calendar)
    }
    #[test]
    fn unknown() {
        assert_eq!(detect_intent("explain quantum tunneling"), Intent::Unknown)
    }
}
