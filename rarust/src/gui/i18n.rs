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
    Name,
    Size,
    Ratio,
    Modified,
    Crc32,
    Method,
    Encrypted,
    SelectFile,
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

    // -- Password dialog --
    ArchivePassword,
    Password,
    ShowPassword,
    RememberSession,
    PasswordCannotBeEmpty,

    // -- Progress dialog --
    Cancel,

    // -- Preview --
    Preview,
    ImagePreviewNotSupported,

    // -- Context menu & actions --
    CloseArchive,
    CreateArchive,
    SelectAll,
    CopyPath,
    SortBy,
    SortByName,
    SortBySize,
    SortByDate,
    SortByRatio,
    SortByMethod,
    SortByCrc,
    SortAscending,
    SortDescending,

    // -- Theme --
    Theme,
    LightTheme,
    DarkTheme,

    // -- Tabs --
    NewTab,
    CloseTab,

    // -- Archive creation --
    ArchiveName,
    CompressionMethod,
    CompressionLevel,
    SplitVolumes,
    AddPassword,
    ConfirmPassword,
    PasswordsDoNotMatch,
    Browse,
    Create,
    OverwriteConfirm,
    OverwriteConfirmDetail,
    NoFilesSelected,
    PasswordProtected,
    EnterPassword,
    WrongPassword,
    FileAlreadyExists,
    ProgressExtracting,
    ProgressTesting,
    ProgressCreating,
    ProgressCompressing,
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
        (Locale::En, Message::Name) => "Name",
        (Locale::En, Message::Size) => "Size",
        (Locale::En, Message::Ratio) => "Ratio",
        (Locale::En, Message::Modified) => "Modified",
        (Locale::En, Message::Crc32) => "CRC32",
        (Locale::En, Message::Method) => "Method",
        (Locale::En, Message::Encrypted) => "Encrypted",
        (Locale::En, Message::SelectFile) => "Select a file to view details",
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
        (Locale::En, Message::ArchivePassword) => "Archive Password",
        (Locale::En, Message::Password) => "Password",
        (Locale::En, Message::ShowPassword) => "Show password",
        (Locale::En, Message::RememberSession) => "Remember for this session",
        (Locale::En, Message::PasswordCannotBeEmpty) => "Password cannot be empty",
        (Locale::En, Message::Cancel) => "Cancel",
        (Locale::En, Message::Preview) => "Preview",
        (Locale::En, Message::ImagePreviewNotSupported) => "(Image preview not yet supported)",
        (Locale::En, Message::CloseArchive) => "Close Archive",
        (Locale::En, Message::CreateArchive) => "New Archive…",
        (Locale::En, Message::SelectAll) => "Select All",
        (Locale::En, Message::CopyPath) => "Copy Path",
        (Locale::En, Message::SortBy) => "Sort by",
        (Locale::En, Message::SortByName) => "Name",
        (Locale::En, Message::SortBySize) => "Size",
        (Locale::En, Message::SortByDate) => "Date",
        (Locale::En, Message::SortByRatio) => "Ratio",
        (Locale::En, Message::SortByMethod) => "Method",
        (Locale::En, Message::SortByCrc) => "CRC32",
        (Locale::En, Message::SortAscending) => "Ascending",
        (Locale::En, Message::SortDescending) => "Descending",
        (Locale::En, Message::Theme) => "Theme",
        (Locale::En, Message::LightTheme) => "Light",
        (Locale::En, Message::DarkTheme) => "Dark",
        (Locale::En, Message::NewTab) => "New Tab",
        (Locale::En, Message::CloseTab) => "Close Tab",
        (Locale::En, Message::ArchiveName) => "Archive name",
        (Locale::En, Message::CompressionMethod) => "Compression method",
        (Locale::En, Message::CompressionLevel) => "Compression level",
        (Locale::En, Message::SplitVolumes) => "Split into volumes (MB)",
        (Locale::En, Message::AddPassword) => "Set password",
        (Locale::En, Message::ConfirmPassword) => "Confirm password",
        (Locale::En, Message::PasswordsDoNotMatch) => "Passwords do not match",
        (Locale::En, Message::Browse) => "Browse…",
        (Locale::En, Message::Create) => "Create",
        (Locale::En, Message::OverwriteConfirm) => "File already exists",
        (Locale::En, Message::OverwriteConfirmDetail) => "Do you want to overwrite it?",
        (Locale::En, Message::NoFilesSelected) => "No files selected",
        (Locale::En, Message::PasswordProtected) => "Password protected",
        (Locale::En, Message::EnterPassword) => "Enter password",
        (Locale::En, Message::WrongPassword) => "Incorrect password",
        (Locale::En, Message::FileAlreadyExists) => "File already exists",
        (Locale::En, Message::ProgressExtracting) => "Extracting…",
        (Locale::En, Message::ProgressTesting) => "Testing…",
        (Locale::En, Message::ProgressCreating) => "Creating archive…",
        (Locale::En, Message::ProgressCompressing) => "Compressing…",

        // --- Simplified Chinese ---
        (Locale::ZhHans, Message::AppTitle) => "Rarust — 归档浏览器",
        (Locale::ZhHans, Message::Archive) => "归档文件",
        (Locale::ZhHans, Message::Format) => "格式",
        (Locale::ZhHans, Message::Name) => "名称",
        (Locale::ZhHans, Message::Size) => "大小",
        (Locale::ZhHans, Message::Ratio) => "压缩率",
        (Locale::ZhHans, Message::Modified) => "修改时间",
        (Locale::ZhHans, Message::Crc32) => "CRC32",
        (Locale::ZhHans, Message::Method) => "压缩方式",
        (Locale::ZhHans, Message::Encrypted) => "加密",
        (Locale::ZhHans, Message::SelectFile) => "选择文件以查看详细信息",
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
        (Locale::ZhHans, Message::ArchivePassword) => "归档密码",
        (Locale::ZhHans, Message::Password) => "密码",
        (Locale::ZhHans, Message::ShowPassword) => "显示密码",
        (Locale::ZhHans, Message::RememberSession) => "本次会话记住",
        (Locale::ZhHans, Message::PasswordCannotBeEmpty) => "密码不能为空",
        (Locale::ZhHans, Message::Cancel) => "取消",
        (Locale::ZhHans, Message::Preview) => "预览",
        (Locale::ZhHans, Message::ImagePreviewNotSupported) => "（图片预览暂不支持）",
        (Locale::ZhHans, Message::CloseArchive) => "关闭归档",
        (Locale::ZhHans, Message::CreateArchive) => "新建归档…",
        (Locale::ZhHans, Message::SelectAll) => "全选",
        (Locale::ZhHans, Message::CopyPath) => "复制路径",
        (Locale::ZhHans, Message::SortBy) => "排序方式",
        (Locale::ZhHans, Message::SortByName) => "名称",
        (Locale::ZhHans, Message::SortBySize) => "大小",
        (Locale::ZhHans, Message::SortByDate) => "日期",
        (Locale::ZhHans, Message::SortByRatio) => "压缩率",
        (Locale::ZhHans, Message::SortByMethod) => "压缩方式",
        (Locale::ZhHans, Message::SortByCrc) => "CRC32",
        (Locale::ZhHans, Message::SortAscending) => "升序",
        (Locale::ZhHans, Message::SortDescending) => "降序",
        (Locale::ZhHans, Message::Theme) => "主题",
        (Locale::ZhHans, Message::LightTheme) => "浅色",
        (Locale::ZhHans, Message::DarkTheme) => "深色",
        (Locale::ZhHans, Message::NewTab) => "新建标签页",
        (Locale::ZhHans, Message::CloseTab) => "关闭标签页",
        (Locale::ZhHans, Message::ArchiveName) => "归档名称",
        (Locale::ZhHans, Message::CompressionMethod) => "压缩方式",
        (Locale::ZhHans, Message::CompressionLevel) => "压缩级别",
        (Locale::ZhHans, Message::SplitVolumes) => "分卷大小（MB）",
        (Locale::ZhHans, Message::AddPassword) => "设置密码",
        (Locale::ZhHans, Message::ConfirmPassword) => "确认密码",
        (Locale::ZhHans, Message::PasswordsDoNotMatch) => "两次密码不一致",
        (Locale::ZhHans, Message::Browse) => "浏览…",
        (Locale::ZhHans, Message::Create) => "创建",
        (Locale::ZhHans, Message::OverwriteConfirm) => "文件已存在",
        (Locale::ZhHans, Message::OverwriteConfirmDetail) => "是否覆盖？",
        (Locale::ZhHans, Message::NoFilesSelected) => "未选择文件",
        (Locale::ZhHans, Message::PasswordProtected) => "密码保护",
        (Locale::ZhHans, Message::EnterPassword) => "输入密码",
        (Locale::ZhHans, Message::WrongPassword) => "密码错误",
        (Locale::ZhHans, Message::FileAlreadyExists) => "文件已存在",
        (Locale::ZhHans, Message::ProgressExtracting) => "正在解压…",
        (Locale::ZhHans, Message::ProgressTesting) => "正在测试…",
        (Locale::ZhHans, Message::ProgressCreating) => "正在创建归档…",
        (Locale::ZhHans, Message::ProgressCompressing) => "正在压缩…",

        // --- Traditional Chinese ---
        (Locale::ZhHant, Message::AppTitle) => "Rarust — 封存瀏覽器",
        (Locale::ZhHant, Message::Archive) => "封存檔",
        (Locale::ZhHant, Message::Format) => "格式",
        (Locale::ZhHant, Message::Name) => "名稱",
        (Locale::ZhHant, Message::Size) => "大小",
        (Locale::ZhHant, Message::Ratio) => "壓縮率",
        (Locale::ZhHant, Message::Modified) => "修改時間",
        (Locale::ZhHant, Message::Crc32) => "CRC32",
        (Locale::ZhHant, Message::Method) => "壓縮方式",
        (Locale::ZhHant, Message::Encrypted) => "加密",
        (Locale::ZhHant, Message::SelectFile) => "選擇檔案以檢視詳細資訊",
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
        (Locale::ZhHant, Message::ArchivePassword) => "封存密碼",
        (Locale::ZhHant, Message::Password) => "密碼",
        (Locale::ZhHant, Message::ShowPassword) => "顯示密碼",
        (Locale::ZhHant, Message::RememberSession) => "本次連線記住",
        (Locale::ZhHant, Message::PasswordCannotBeEmpty) => "密碼不可為空",
        (Locale::ZhHant, Message::Cancel) => "取消",
        (Locale::ZhHant, Message::Preview) => "預覽",
        (Locale::ZhHant, Message::ImagePreviewNotSupported) => "（圖片預覽暫不支援）",
        (Locale::ZhHant, Message::CloseArchive) => "關閉封存",
        (Locale::ZhHant, Message::CreateArchive) => "新建封存…",
        (Locale::ZhHant, Message::SelectAll) => "全選",
        (Locale::ZhHant, Message::CopyPath) => "複製路徑",
        (Locale::ZhHant, Message::SortBy) => "排序方式",
        (Locale::ZhHant, Message::SortByName) => "名稱",
        (Locale::ZhHant, Message::SortBySize) => "大小",
        (Locale::ZhHant, Message::SortByDate) => "日期",
        (Locale::ZhHant, Message::SortByRatio) => "壓縮率",
        (Locale::ZhHant, Message::SortByMethod) => "壓縮方式",
        (Locale::ZhHant, Message::SortByCrc) => "CRC32",
        (Locale::ZhHant, Message::SortAscending) => "升序",
        (Locale::ZhHant, Message::SortDescending) => "降序",
        (Locale::ZhHant, Message::Theme) => "主題",
        (Locale::ZhHant, Message::LightTheme) => "淺色",
        (Locale::ZhHant, Message::DarkTheme) => "深色",
        (Locale::ZhHant, Message::NewTab) => "新分頁",
        (Locale::ZhHant, Message::CloseTab) => "關閉分頁",
        (Locale::ZhHant, Message::ArchiveName) => "封存名稱",
        (Locale::ZhHant, Message::CompressionMethod) => "壓縮方式",
        (Locale::ZhHant, Message::CompressionLevel) => "壓縮級別",
        (Locale::ZhHant, Message::SplitVolumes) => "分卷大小（MB）",
        (Locale::ZhHant, Message::AddPassword) => "設定密碼",
        (Locale::ZhHant, Message::ConfirmPassword) => "確認密碼",
        (Locale::ZhHant, Message::PasswordsDoNotMatch) => "兩次密碼不一致",
        (Locale::ZhHant, Message::Browse) => "瀏覽…",
        (Locale::ZhHant, Message::Create) => "建立",
        (Locale::ZhHant, Message::OverwriteConfirm) => "檔案已存在",
        (Locale::ZhHant, Message::OverwriteConfirmDetail) => "是否覆蓋？",
        (Locale::ZhHant, Message::NoFilesSelected) => "未選擇檔案",
        (Locale::ZhHant, Message::PasswordProtected) => "密碼保護",
        (Locale::ZhHant, Message::EnterPassword) => "輸入密碼",
        (Locale::ZhHant, Message::WrongPassword) => "密碼錯誤",
        (Locale::ZhHant, Message::FileAlreadyExists) => "檔案已存在",
        (Locale::ZhHant, Message::ProgressExtracting) => "正在解壓…",
        (Locale::ZhHant, Message::ProgressTesting) => "正在測試…",
        (Locale::ZhHant, Message::ProgressCreating) => "正在建立封存…",
        (Locale::ZhHant, Message::ProgressCompressing) => "正在壓縮…",
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
            Message::Name,
            Message::Size,
            Message::Ratio,
            Message::Modified,
            Message::Crc32,
            Message::Method,
            Message::Encrypted,
            Message::SelectFile,
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
            Message::ArchivePassword,
            Message::Password,
            Message::ShowPassword,
            Message::RememberSession,
            Message::PasswordCannotBeEmpty,
            Message::Cancel,
            Message::Preview,
            Message::ImagePreviewNotSupported,
            Message::CloseArchive,
            Message::CreateArchive,
            Message::SelectAll,
            Message::CopyPath,
            Message::SortBy,
            Message::SortByName,
            Message::SortBySize,
            Message::SortByDate,
            Message::SortByRatio,
            Message::SortByMethod,
            Message::SortByCrc,
            Message::SortAscending,
            Message::SortDescending,
            Message::Theme,
            Message::LightTheme,
            Message::DarkTheme,
            Message::NewTab,
            Message::CloseTab,
            Message::ArchiveName,
            Message::CompressionMethod,
            Message::CompressionLevel,
            Message::SplitVolumes,
            Message::AddPassword,
            Message::ConfirmPassword,
            Message::PasswordsDoNotMatch,
            Message::Browse,
            Message::Create,
            Message::OverwriteConfirm,
            Message::OverwriteConfirmDetail,
            Message::NoFilesSelected,
            Message::PasswordProtected,
            Message::EnterPassword,
            Message::WrongPassword,
            Message::FileAlreadyExists,
            Message::ProgressExtracting,
            Message::ProgressTesting,
            Message::ProgressCreating,
            Message::ProgressCompressing,
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
