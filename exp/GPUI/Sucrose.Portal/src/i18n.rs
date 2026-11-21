use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Tr,
    En,
}

static ENGLISH: AtomicBool = AtomicBool::new(false);

pub fn set_lang(lang: Lang) {
    ENGLISH.store(lang == Lang::En, Ordering::Relaxed);
}

pub fn lang() -> Lang {
    if ENGLISH.load(Ordering::Relaxed) {
        Lang::En
    } else {
        Lang::Tr
    }
}

/// (key, Turkish, English)
const STRINGS: &[(&str, &str, &str)] = &[
    ("library", "Kütüphane", "Library"),
    ("store", "Mağaza", "Store"),
    ("search", "Duvar kağıdı ara", "Search for a Wallpaper"),
    ("settings", "Ayarlar", "Settings"),
    ("back", "Geri", "Back"),
    ("toggle_theme", "Temayı değiştir", "Toggle theme"),
    ("create", "Tema oluştur", "Create theme"),
    ("pause_wallpaper", "Duvar kağıdını durdur", "Pause wallpaper"),
    ("play_wallpaper", "Duvar kağıdını başlat", "Play wallpaper"),
    ("display", "Ekran tercihleri", "Display preferences"),
    ("more", "Diğer", "More"),
    ("cycling", "Duvar kağıdı döngüsü", "Wallpaper cycling"),
    ("refresh_library", "Kütüphaneyi yenile", "Refresh library"),
    ("report_bug", "Hata bildir", "Report a bug"),
    ("use", "Kullan", "Use"),
    ("customize", "Özelleştir", "Customize"),
    ("edit", "Düzenle", "Edit"),
    ("show_in_folder", "Klasörde göster", "Show in folder"),
    ("share", "Paylaş", "Share"),
    ("report", "Bildir", "Report"),
    ("delete", "Listeden kaldır", "Remove from list"),
    ("removed_note", "Prototip: yalnızca listeden kaldırıldı, dosyalara dokunulmadı.", "Prototype: removed from the list only, files were not touched."),
    ("not_in_prototype", "Bu özellik prototipte henüz yok.", "This feature is not in the prototype yet."),
    ("active", "Aktif", "Active"),
    ("library_empty", "Kütüphanen boş", "Your library is empty"),
    ("no_results", "Sonuç bulunamadı", "No results found"),
    ("store_unavailable", "Mağaza önbelleği bulunamadı", "Store cache not found"),
    ("all", "Tümü", "All"),
    ("download", "İndir", "Download"),
    ("in_library", "Kütüphanede", "In library"),
    ("download_done_note", "Prototip: indirme simüle edildi.", "Prototype: download was simulated."),
    ("general", "Genel", "General"),
    ("personal", "Kişisel", "Personal"),
    ("system", "Sistem", "System"),
    ("wallpaper", "Duvar Kağıdı", "Wallpaper"),
    ("performance", "Performans", "Performance"),
    ("other", "Diğer", "Other"),
    ("donate", "Bağış", "Donate"),
    ("language", "Uygulama Dili", "Application Language"),
    ("language_desc", "Uygulamayı görüntülemek istediğiniz dili seçin.", "Select the language in which you want to view the application."),
    ("theme", "Uygulama Teması", "Application Theme"),
    ("theme_desc", "Arayüzün koyu veya açık görünümü.", "Dark or light appearance of the interface."),
    ("dark", "Koyu", "Dark"),
    ("light", "Açık", "Light"),
    ("backdrop", "Pencere Arka Planı", "Window Backdrop"),
    ("backdrop_desc", "Uygulama penceresinin arka planını özelleştirmek için kullanılır.", "Used to customize the background of the application window."),
    ("backdrop_opacity", "Arka Plan Opaklığı (%)", "Background Opacity (%)"),
    ("acrylic", "Akrilik", "Acrylic"),
    ("solid", "Düz", "Solid"),
    ("startup", "Uygulama Başlangıcı", "Startup at Application Launch"),
    ("startup_desc", "Sucrose'un duvar kağıdını oynatması için başlangıçta arka planda çalışması gerekir.", "Sucrose needs to run in the background at startup to play the wallpaper."),
    ("none", "Yok", "None"),
    ("priority", "Öncelikli", "Priority"),
    ("scheduler", "Zamanlayıcı", "Scheduler"),
    ("tray", "Bildirim Alanı Simgesi", "Notification Area Icon"),
    ("tray_desc", "Sistem tepsisindeki görünürlük. Sucrose sessizce çalışmaya devam eder.", "Visibility in the system tray. Sucrose will continue to run discreetly."),
    ("visible", "Görünür", "Visible"),
    ("invisible", "Görünmez", "Invisible"),
    ("tray_exit", "Sistem tepsisi kapatıldığında tüm Sucrose uygulamalarını kapat", "Close all Sucrose applications when the system tray is closed"),
    ("volume", "Motor Ses Seviyesi", "Engine Volume"),
    ("volume_desc", "Tüm duvar kağıtlarına uygulanan ses seviyesi.", "Volume that applies to all wallpapers."),
    ("volume_active", "Yalnızca tüm uygulamalar sessizken ses çal", "Play sound only when all applications are silent"),
    ("volume_desktop", "Yalnızca masaüstüne odaklanıldığında ses çal", "Play sound only when focused on the desktop"),
    ("library_location", "Kütüphane Konumu", "Library Location"),
    ("library_location_desc", "Kütüphanene eklediğin temaların saklandığı konum.", "Location where themes you add to your library are stored."),
    ("change", "Değiştir", "Change"),
    ("library_move", "Mevcut kütüphaneni yeni konuma taşı", "Move your current library to the new location"),
    ("library_preview", "Kütüphane Önizlemesi", "Library Preview"),
    ("library_preview_desc", "Kartın üzerine gelindiğinde hareketli önizlemeyi oynat.", "Play the animated preview when hovering over a card."),
    ("library_preview_toggle", "Önizlemeyi etkinleştir", "Enable preview"),
    ("cpu", "İşlemci Kullanımı", "CPU Usage"),
    ("cpu_desc", "İşlemci kullanımı bu değeri aştığında duvar kağıdını durdur.", "Pause the wallpaper when CPU usage exceeds this value."),
    ("memory", "Bellek Kullanımı", "Memory Usage"),
    ("memory_desc", "Bellek kullanımı bu değeri aştığında duvar kağıdını durdur.", "Pause the wallpaper when memory usage exceeds this value."),
    ("battery", "Pil Seviyesi", "Battery Level"),
    ("battery_desc", "Pil seviyesi bu değerin altına düştüğünde duvar kağıdını durdur.", "Pause the wallpaper when the battery drops below this value."),
    ("fullscreen", "Tam Ekran", "Fullscreen"),
    ("fullscreen_desc", "Tam ekran bir uygulama açıkken yapılacak işlem.", "What to do while a fullscreen application is open."),
    ("pause", "Durdur", "Pause"),
    ("close", "Kapat", "Close"),
    ("continue", "Devam et", "Continue"),
    ("remote", "Uzak Masaüstü", "Remote Desktop"),
    ("remote_desc", "Uzak masaüstü bağlantısı sırasında duvar kağıdını durdur.", "Pause the wallpaper during a remote desktop session."),
    ("saver", "Pil Tasarrufu", "Battery Saver"),
    ("saver_desc", "Pil tasarrufu modu açıkken duvar kağıdını durdur.", "Pause the wallpaper while battery saver is on."),
    ("enabled", "Etkin", "Enabled"),
    ("about", "Hakkında", "About"),
    ("about_desc", "Sucrose Portal arayüzünün GPUI ile yazılmış deneme sürümü.", "Experimental build of the Sucrose Portal UI written with GPUI."),
    ("about_ui", "Arayüz", "Interface"),
    ("about_language", "Dil", "Language"),
    ("about_platform", "Platform", "Platform"),
    ("about_memory", "Bellek", "Memory"),
    ("section_soon", "Bu bölüm prototipte henüz yok.", "This section is not in the prototype yet."),
    ("prerelease", "Ön Sürüm", "Pre-Release"),
    ("copyright", "Sucrose © 2026 Sucrose Katkıda Bulunanlar", "Sucrose is copyright © 2026 Sucrose Contributors"),
];

pub fn t(key: &str) -> &'static str {
    let english = ENGLISH.load(Ordering::Relaxed);
    STRINGS
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, tr, en)| if english { *en } else { *tr })
        .unwrap_or("???")
}

const QUOTES: &[(&str, &str)] = &[
    ("Söz gümüşse, sükût altındır", "Speech is silver, silence is golden"),
    ("Damlaya damlaya göl olur", "Many drops make a lake"),
    ("Sabır acıdır, meyvesi tatlıdır", "Patience is bitter, but its fruit is sweet"),
    ("Az olsun, öz olsun", "Less, but better"),
    ("Bilgi güçtür", "Knowledge is power"),
];

pub fn quote(index: usize) -> &'static str {
    let (tr, en) = QUOTES[index % QUOTES.len()];
    if lang() == Lang::En { en } else { tr }
}
