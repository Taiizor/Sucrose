using SMMRU = Sucrose.Memory.Manage.Readonly.Url;
using SMMRGH = Sucrose.Memory.Manage.Readonly.GitHub;
using SMMRS = Sucrose.Memory.Manage.Readonly.Soferity;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRF = Sucrose.Memory.Manage.Readonly.Folder;
using SMMRC = Sucrose.Memory.Manage.Readonly.Content;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;

namespace Sucrose.Memory
{
    public static class Readonly
    {
        public static readonly string Bundle = "Bundle";

        public static readonly string Themes = "Themes";

        public static readonly string Key = string.Empty;

        public static readonly string Default = "Default";

        public static readonly string Library = "Library";

        public static readonly string StoreSource = "src";

        public static readonly string Unknown = "Unknown";

        public static readonly string PipeServerName = ".";

        public static readonly string Showcase = "Showcase";

        public static readonly string SoferityFile = "File";

        public static readonly char StartCommandChar = '✔';

        public static readonly string SoferityVersion = "v6";

        public static readonly string ExceptionSplit = " -> ";

        public static readonly string SoferityCheck = "Check";

        public static readonly string SoferityError = "Error";

        public static readonly string SoferityStore = "Store";

        public static readonly string SoferityTheme = "Theme";

        public static readonly char ValueSeparatorChar = '✖';

        public static readonly string StoreFile = "Store.json";

        public static readonly string SoferityOnline = "Online";

        public static readonly string SoferityReport = "Report";

        public static readonly string SoferitySearch = "Search";

        public static readonly string SoferityUpdate = "Update";

        public static readonly string SoferityUpload = "Upload";

        public static readonly string SoferityPattern = "Pattern";

        public static readonly string HostEntry = "www.google.com";

        public static readonly string PatternFile = "Pattern.json";

        public static readonly string SoferityFeedback = "Feedback";

        public static readonly string SoferityStatistic = "Statistic";

        public static readonly string TaskName = "Autorun for Sucrose";

        public static readonly string SucroseInfo = "SucroseInfo.json";

        public static readonly string StartCommand = $"{StartCommandChar}";

        public static readonly string WebViewProcessName = "msedgewebview2";

        public static readonly string ValueSeparator = $"{ValueSeparatorChar}";

        public static readonly string LogDescription = "SucroseWatchdog Thread";

        public static readonly string DiscordApplication = "1126294965950103612";

        public static readonly string TaskDescription = "Sucrose Wallpaper Engine";

        public static readonly string SucroseCompatible = "SucroseCompatible.json";

        public static readonly string SucroseProperties = "SucroseProperties.json";

        public static readonly string Guid = "00000000-0000-0000-0000-000000000000";

        public static readonly string CefSharpProcessName = "CefSharp.BrowserSubprocess";

        public static readonly string UserAgent = "Sucrose/1.8 (Wallpaper Engine) SucroseWebKit";
    }
}