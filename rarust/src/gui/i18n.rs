//! Internationalization for the egui GUI.
//!
//! Supports English, Simplified Chinese, and Traditional Chinese with
//! runtime locale switching.

use std::str::FromStr;

/// Supported UI locales.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Locale {
    /// English.
    En,
    /// Simplified Chinese (简体中文).
    ZhHans,
    /// Traditional Chinese (繁體中文).
    ZhHant,
}

impl Locale {
    /// All locales available in the language picker.
    pub const ALL: [Locale; 3] = [Locale::En, Locale::ZhHans, Locale::ZhHant];

    /// Human-readable name shown in the language selector.
    pub fn display_name(self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::ZhHans => "简体中文",
            Locale::ZhHant => "繁體中文",
        }
    }

    /// Detect locale from `RARUST_LANG` or system environment.
    pub fn detect() -> Self {
        if let Ok(lang) = std::env::var("RARUST_LANG")
            && let Ok(locale) = lang.parse()
        {
            return locale;
        }
        detect_system_locale()
    }

    /// Whether this locale needs a CJK-capable font.
    pub fn needs_cjk_font(self) -> bool {
        matches!(self, Locale::ZhHans | Locale::ZhHant)
    }
}

impl FromStr for Locale {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace('_', "-").as_str() {
            "en" | "en-us" | "en-gb" => Ok(Locale::En),
            "zh" | "zh-cn" | "zh-hans" | "zh-sg" => Ok(Locale::ZhHans),
            "zh-tw" | "zh-hk" | "zh-mo" | "zh-hant" => Ok(Locale::ZhHant),
            _ => Err(()),
        }
    }
}

/// Translation message keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Message {
    AppTitle,
    Archive,
    Format,
    Entries,
    Name,
    Size,
    Ratio,
    Modified,
    Crc32,
    Method,
    Encrypted,
    Directory,
    Yes,
    No,
    SelectFile,
    SelectFileDetail,
    TotalSize,
    SearchPlaceholder,
    Extract,
    Test,
    Refresh,
    Language,
    StatusReady,
    StatusLoading,
    StatusError,
    EmptyArchive,
    FilesCount,
    FontWarning,
    FontWarningDetail,
    OpenArchive,
    Welcome,
    WelcomeDetail,
}

/// Active locale with translation helpers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct I18n {
    locale: Locale,
}

impl I18n {
    /// Create an i18n context for the given locale.
    pub fn new(locale: Locale) -> Self {
        Self { locale }
    }

    /// Current locale.
    pub fn locale(self) -> Locale {
        self.locale
    }

    /// Switch locale.
    pub fn set_locale(&mut self, locale: Locale) {
        self.locale = locale;
    }

    /// Translate a message key.
    pub fn t(self, msg: Message) -> &'static str {
        translate(self.locale, msg)
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new(Locale::detect())
    }
}

fn translate(locale: Locale, msg: Message) -> &'static str {
    match (locale, msg) {
        // --- English ---
        (Locale::En, Message::AppTitle) => "Rarust — Archive Browser",
        (Locale::En, Message::Archive) => "Archive",
        (Locale::En, Message::Format) => "Format",
        (Locale::En, Message::Entries) => "Entries",
        (Locale::En, Message::Name) => "Name",
        (Locale::En, Message::Size) => "Size",
        (Locale::En, Message::Ratio) => "Ratio",
        (Locale::En, Message::Modified) => "Modified",
        (Locale::En, Message::Crc32) => "CRC32",
        (Locale::En, Message::Method) => "Method",
        (Locale::En, Message::Encrypted) => "Encrypted",
        (Locale::En, Message::Directory) => "Directory",
        (Locale::En, Message::Yes) => "Yes",
        (Locale::En, Message::No) => "No",
        (Locale::En, Message::SelectFile) => "Select a file to view details",
        (Locale::En, Message::SelectFileDetail) => {
            "Choose an entry from the archive list to inspect metadata and actions."
        }
        (Locale::En, Message::TotalSize) => "Total size",
        (Locale::En, Message::SearchPlaceholder) => "Search files…",
        (Locale::En, Message::Extract) => "Extract",
        (Locale::En, Message::Test) => "Test",
        (Locale::En, Message::Refresh) => "Refresh",
        (Locale::En, Message::Language) => "Language",
        (Locale::En, Message::StatusReady) => "Ready",
        (Locale::En, Message::StatusLoading) => "Loading archive…",
        (Locale::En, Message::StatusError) => "Error",
        (Locale::En, Message::EmptyArchive) => "(empty archive)",
        (Locale::En, Message::FilesCount) => "files",
        (Locale::En, Message::FontWarning) => "CJK font not found",
        (Locale::En, Message::FontWarningDetail) => {
            "Chinese characters may not display correctly. Install a CJK font or set RARUST_FONT."
        }
        (Locale::En, Message::OpenArchive) => "Open…",
        (Locale::En, Message::Welcome) => "Welcome to Rarust",
        (Locale::En, Message::WelcomeDetail) => {
            "Open a RAR, ZIP, TAR, TAR.GZ, TGZ, or GZ archive to browse its contents."
        }

        // --- Simplified Chinese ---
        (Locale::ZhHans, Message::AppTitle) => "Rarust — 归档浏览器",
        (Locale::ZhHans, Message::Archive) => "归档文件",
        (Locale::ZhHans, Message::Format) => "格式",
        (Locale::ZhHans, Message::Entries) => "项目",
        (Locale::ZhHans, Message::Name) => "名称",
        (Locale::ZhHans, Message::Size) => "大小",
        (Locale::ZhHans, Message::Ratio) => "压缩率",
        (Locale::ZhHans, Message::Modified) => "修改时间",
        (Locale::ZhHans, Message::Crc32) => "CRC32",
        (Locale::ZhHans, Message::Method) => "压缩方式",
        (Locale::ZhHans, Message::Encrypted) => "加密",
        (Locale::ZhHans, Message::Directory) => "目录",
        (Locale::ZhHans, Message::Yes) => "是",
        (Locale::ZhHans, Message::No) => "否",
        (Locale::ZhHans, Message::SelectFile) => "选择文件以查看详细信息",
        (Locale::ZhHans, Message::SelectFileDetail) => {
            "从归档列表中选择项目，以查看元数据和可用操作。"
        }
        (Locale::ZhHans, Message::TotalSize) => "总大小",
        (Locale::ZhHans, Message::SearchPlaceholder) => "搜索文件…",
        (Locale::ZhHans, Message::Extract) => "解压",
        (Locale::ZhHans, Message::Test) => "测试",
        (Locale::ZhHans, Message::Refresh) => "刷新",
        (Locale::ZhHans, Message::Language) => "语言",
        (Locale::ZhHans, Message::StatusReady) => "就绪",
        (Locale::ZhHans, Message::StatusLoading) => "正在加载归档…",
        (Locale::ZhHans, Message::StatusError) => "错误",
        (Locale::ZhHans, Message::EmptyArchive) => "（空归档）",
        (Locale::ZhHans, Message::FilesCount) => "个文件",
        (Locale::ZhHans, Message::FontWarning) => "找不到 CJK 字体",
        (Locale::ZhHans, Message::FontWarningDetail) => {
            "中文可能无法正确显示。请安装 CJK 字体或设置 RARUST_FONT 环境变量。"
        }
        (Locale::ZhHans, Message::OpenArchive) => "打开…",
        (Locale::ZhHans, Message::Welcome) => "欢迎使用 Rarust",
        (Locale::ZhHans, Message::WelcomeDetail) => {
            "打开 RAR、ZIP、TAR、TAR.GZ、TGZ 或 GZ 归档以浏览内容。"
        }

        // --- Traditional Chinese ---
        (Locale::ZhHant, Message::AppTitle) => "Rarust — 封存瀏覽器",
        (Locale::ZhHant, Message::Archive) => "封存檔",
        (Locale::ZhHant, Message::Format) => "格式",
        (Locale::ZhHant, Message::Entries) => "項目",
        (Locale::ZhHant, Message::Name) => "名稱",
        (Locale::ZhHant, Message::Size) => "大小",
        (Locale::ZhHant, Message::Ratio) => "壓縮率",
        (Locale::ZhHant, Message::Modified) => "修改時間",
        (Locale::ZhHant, Message::Crc32) => "CRC32",
        (Locale::ZhHant, Message::Method) => "壓縮方式",
        (Locale::ZhHant, Message::Encrypted) => "加密",
        (Locale::ZhHant, Message::Directory) => "目錄",
        (Locale::ZhHant, Message::Yes) => "是",
        (Locale::ZhHant, Message::No) => "否",
        (Locale::ZhHant, Message::SelectFile) => "選擇檔案以檢視詳細資訊",
        (Locale::ZhHant, Message::SelectFileDetail) => {
            "從封存清單中選擇項目，以檢視中繼資料與可用操作。"
        }
        (Locale::ZhHant, Message::TotalSize) => "總大小",
        (Locale::ZhHant, Message::SearchPlaceholder) => "搜尋檔案…",
        (Locale::ZhHant, Message::Extract) => "解壓",
        (Locale::ZhHant, Message::Test) => "測試",
        (Locale::ZhHant, Message::Refresh) => "重新整理",
        (Locale::ZhHant, Message::Language) => "語言",
        (Locale::ZhHant, Message::StatusReady) => "就緒",
        (Locale::ZhHant, Message::StatusLoading) => "正在載入封存…",
        (Locale::ZhHant, Message::StatusError) => "錯誤",
        (Locale::ZhHant, Message::EmptyArchive) => "（空封存）",
        (Locale::ZhHant, Message::FilesCount) => "個檔案",
        (Locale::ZhHant, Message::FontWarning) => "找不到中日韓字型",
        (Locale::ZhHant, Message::FontWarningDetail) => {
            "中文可能無法正確顯示。請安裝 CJK 字型或設定 RARUST_FONT 環境變數。"
        }
        (Locale::ZhHant, Message::OpenArchive) => "開啟…",
        (Locale::ZhHant, Message::Welcome) => "歡迎使用 Rarust",
        (Locale::ZhHant, Message::WelcomeDetail) => {
            "開啟 RAR、ZIP、TAR、TAR.GZ、TGZ 或 GZ 封存以瀏覽內容。"
        }
    }
}

/// Best-effort system locale detection without external crates.
fn detect_system_locale() -> Locale {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(key) {
            let tag = val.split('.').next().unwrap_or(&val);
            if let Ok(locale) = tag.parse::<Locale>() {
                return locale;
            }
        }
    }
    Locale::En
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_from_str_parses_common_tags() {
        assert_eq!("en".parse(), Ok(Locale::En));
        assert_eq!("zh-CN".parse(), Ok(Locale::ZhHans));
        assert_eq!("zh_TW".parse(), Ok(Locale::ZhHant));
    }

    #[test]
    fn all_messages_have_translations() {
        let keys = [
            Message::AppTitle,
            Message::Archive,
            Message::Format,
            Message::Entries,
            Message::Name,
            Message::Size,
            Message::Ratio,
            Message::Modified,
            Message::Crc32,
            Message::Method,
            Message::Encrypted,
            Message::Directory,
            Message::Yes,
            Message::No,
            Message::SelectFile,
            Message::SelectFileDetail,
            Message::TotalSize,
            Message::SearchPlaceholder,
            Message::Extract,
            Message::Test,
            Message::Refresh,
            Message::Language,
            Message::StatusReady,
            Message::StatusLoading,
            Message::StatusError,
            Message::EmptyArchive,
            Message::FilesCount,
            Message::FontWarning,
            Message::FontWarningDetail,
            Message::OpenArchive,
            Message::Welcome,
            Message::WelcomeDetail,
        ];

        for locale in Locale::ALL {
            for key in keys {
                let text = translate(locale, key);
                assert!(
                    !text.is_empty(),
                    "missing translation for {locale:?} {key:?}"
                );
            }
        }
    }

    #[test]
    fn chinese_translations_are_distinct_from_english() {
        let en = I18n::new(Locale::En).t(Message::Extract);
        let zh = I18n::new(Locale::ZhHans).t(Message::Extract);
        assert_ne!(en, zh);
    }
}
