using System;

namespace Sucrose.Memory
{
    public static class Readonly
    {
        public static readonly Random Randomise = new();

        public static readonly string LogFolder = "Log";

        public static readonly string AppName = "Sucrose";

        public static readonly char StartCommandChar = '-';

        public static readonly string CacheFolder = "Cache";

        public static readonly string CefSharp = "CefSharp";

        public static readonly string WebView2 = "WebView2";

        public static readonly char ValueSeparatorChar = '|';

        public static readonly string MediaElement = "MediaElement";

        public static readonly string StartCommand = $"{StartCommandChar}";

        public static readonly string WPFUserInterface = "Sucrose.WPF.UI.exe";

        public static readonly string CommandLine = "Sucrose.CommandLine.exe";

        public static readonly string ValueSeparator = $"{ValueSeparatorChar}";

        public static readonly string LogDescription = "SucroseWatchdog Thread";

        public static readonly string WinFormsUserInterface = "Sucrose.WinForms.UI.exe";

        public static readonly string CefSharpMutex = "{Sucrose-Wallpaper-Engine-Cef-Sharp}";

        public static readonly string TrayIconMutex = "{Sucrose-Wallpaper-Engine-Tray-Icon}";

        public static readonly string MediaElementMutex = "{Sucrose-Wallpaper-Engine-Media-Element}";

        public static readonly string UserInterfaceMutex = "{Sucrose-Wallpaper-Engine-User-Interface}";

        public static readonly string ReportWebsite = "https://github.com/Taiizor/Sucrose/issues/new/choose";

        public static readonly string AppDataPath = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
    }
}