export type Lang = "en" | "ar";
const strings = {
  en: { settings: "Settings", changeCharacter: "Change Character", size: "Size", show: "Show Adam", hide: "Hide Adam", close: "Close", drop: "Drop a GLB/GLTF file here", idle: "Adam is ready" },
  ar: { settings: "الإعدادات", changeCharacter: "تغيير الشخصية", size: "الحجم", show: "إظهار آدم", hide: "إخفاء آدم", close: "إغلاق", drop: "اسحب ملف GLB/GLTF إلى هنا", idle: "آدم جاهز" }
} as const;
export function t(lang: Lang, key: keyof typeof strings.en) { return strings[lang][key]; }