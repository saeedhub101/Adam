use chrono::Local;

pub fn reply(input: &str, language: &str) -> String {
    let q = input.trim().to_lowercase();
    if q.is_empty() { return String::new(); }
    if q.contains("hello") || q.contains("hi") || q.contains("مرحبا") || q.contains("اهلا") {
        return if language == "ar" { "مرحباً، أنا آدم. أنا متاح في الوضع المحلي.".into() } else { "Hello, I’m Adam. I’m available in local mode.".into() };
    }
    if q.contains("time") || q.contains("الوقت") || q.contains("الساعة") {
        let now = Local::now().format("%Y-%m-%d %H:%M").to_string();
        return if language == "ar" { format!("الوقت المحلي الآن: {now}") } else { format!("The local time is {now}.") };
    }
    if q.contains("memory") || q.contains("ذاكرة") {
        return if language == "ar" { "يمكنني البحث في ذاكرتك المحلية وحفظ الملاحظات وتعديلها وحذفها.".into() } else { "I can search local memory and save, edit, or delete notes.".into() };
    }
    if language == "ar" { "لا أستطيع تنفيذ هذه الإجابة بالكامل دون نموذج سحابي، لكن الذاكرة والتذكيرات والصوت المحلي ما زالت تعمل.".into() }
    else { "I can’t fully answer that without a cloud model, but local memory, reminders, and local voice remain available.".into() }
}

#[cfg(test)]
mod tests {
    use super::reply;
    #[test] fn greeting_is_local() { assert!(reply("hello", "en").contains("Adam")); }
    #[test] fn time_intent_is_detected() { assert!(reply("what time is it", "en").contains("time")); }
}
