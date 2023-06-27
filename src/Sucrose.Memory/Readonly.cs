using System;

namespace Sucrose.Memory
{
    public static class Readonly
    {
        public static readonly string AppName = "Sucrose";

        public static readonly char StartCommandChar = '-';

        public static readonly string CacheFolder = "Cache";

        public static readonly string CefSharp = "CefSharp";

        public static readonly char ValueSeparatorChar = '|';

        public static readonly string StartCommand = $"{StartCommandChar}";

        public static readonly string WPFApplication = "Sucrose.WPF.UI.exe";

        public static readonly string ValueSeparator = $"{ValueSeparatorChar}";

        public static readonly string ConsoleApplication = "Sucrose.CommandLine.exe";

        public static readonly string WinFormsApplication = "Sucrose.WinForms.UI.exe";

        public static readonly string TrayIconMutex = "{Sucrose-Wallpaper-Engine-Tray-Icon}";

        public static readonly string ReportWebsite = "https://github.com/Taiizor/Sucrose/issues/new/choose";

        public static readonly string AppDataPath = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
    }
}