namespace Sucrose.Memory
{
    public static class Readonly
    {
        public static readonly Random Randomise = new();

        public static readonly string LogFolder = "Log";

        public static readonly string AppName = "Sucrose";

        public static readonly string Content = "Content";

        public static readonly char StartCommandChar = '-';

        public static readonly string CacheFolder = "Cache";

        public static readonly string CefSharp = "CefSharp";

        public static readonly string WebView2 = "WebView2";

        public static readonly char ValueSeparatorChar = '|';

        public static readonly string MediaElement = "MediaElement";

        public static readonly string AppLive = "Sucrose.Live.AP.exe";

        public static readonly string SucroseInfo = "SucroseInfo.json";

        public static readonly string WPFTrayIcon = "Sucrose.WPF.TI.exe";

        public static readonly string VideoContent = "VideoContent.html";

        public static readonly string WebViewLive = "Sucrose.Live.WV.exe";

        public static readonly string Commandog = "Sucrose.Commandog.exe";

        public static readonly string StartCommand = $"{StartCommandChar}";

        public static readonly string CefSharpLive = "Sucrose.Live.CS.exe";

        public static readonly string LiveMutex = "{Sucrose-Wallpaper-Live}";

        public static readonly string WPFUserInterface = "Sucrose.WPF.UI.exe";

        public static readonly string MediaElementLive = "Sucrose.Live.ME.exe";

        public static readonly string ValueSeparator = $"{ValueSeparatorChar}";

        public static readonly string LogDescription = "SucroseWatchdog Thread";

        public static readonly string DiscordApplication = "1126294965950103612";

        public static readonly string WinFormsTrayIcon = "Sucrose.WinForms.TI.exe";

        public static readonly string SucroseProperties = "SucroseProperties.json";

        public static readonly string WinFormsUserInterface = "Sucrose.WinForms.UI.exe";

        public static readonly string BrowseWebsite = "https://github.com/Taiizor/Sucrose";

        public static readonly string CefSharpMutex = "{Sucrose-Wallpaper-Engine-Cef-Sharp}"; //LiveMutex

        public static readonly string TrayIconMutex = "{Sucrose-Wallpaper-Engine-Tray-Icon}";

        public static readonly string UserInterfaceMutex = "{Sucrose-Wallpaper-Engine-User-Interface}";

        public static readonly string ReportWebsite = "https://github.com/Taiizor/Sucrose/issues/new/choose";

        public static readonly string DownloadWebsite = "https://github.com/Taiizor/Sucrose/releases/latest";

        public static readonly string DocumentsPath = Environment.GetFolderPath(Environment.SpecialFolder.MyDocuments);

        public static readonly string AppDataPath = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
    }
}