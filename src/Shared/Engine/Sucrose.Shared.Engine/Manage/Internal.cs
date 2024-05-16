using System.Windows.Threading;
using SSDSHS = Sucrose.Shared.Dependency.Struct.HandleStruct;
using SSESB = Sucrose.Shared.Engine.Setting.Browser;
using SSTHC = Sucrose.Shared.Theme.Helper.Compatible;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHP = Sucrose.Shared.Theme.Helper.Properties;

namespace Sucrose.Shared.Engine.Manage
{
    internal static class Internal
    {
        public static SSTHI Info = new();

        public static bool Interaction = true;

        public static SSTHP Properties = new();

        public static bool Initialized = false;

        public static SSTHC Compatible = new();

        public static bool PausePerformance = false;

        public static bool PropertiesWatcher = true;

        public static string CpuData = string.Empty;

        public static string BiosData = string.Empty;

        public static string DateData = string.Empty;

        public static string InfoPath = string.Empty;

        public static string AudioData = string.Empty;

        public static string MemoryData = string.Empty;

        public static List<SSDSHS> Applications = new();

        public static string BatteryData = string.Empty;

        public static string NetworkData = string.Empty;

        public static string GraphicData = string.Empty;

        public static string WatcherFile = string.Empty;

        public static IntPtr WindowHandle = IntPtr.Zero;

        public delegate void ExecuteNormal(string Script);

        public static string CompatiblePath = string.Empty;

        public static string PropertiesFile = string.Empty;

        public static string PropertiesPath = string.Empty;

        public static string LibraryLocation = string.Empty;

        public static string LibrarySelected = string.Empty;

        public static string MotherboardData = string.Empty;

        public static string PropertiesCache = string.Empty;

        public delegate Task<string> ExecuteTask(string script);

        public static DateTime DisplayChanged = DateTime.MinValue;

        public static readonly DispatcherTimer GeneralTimer = new();

        public static SSESB BrowserSettings = new()
        {
            WebView = WebArguments,
            CefSharp = CefArguments
        };

        public static List<string> WebArguments = new()
        {
            "--enable-gpu",
            "--enable-gpu-vsync",
            "--disable-gpu-compositing",
            "--disable-direct-write",
            //"--disable-frame-rate-limit",
            "--enable-begin-frame-scheduling",
            "--disable-breakpad",
            "--disable-extensions",

            "--multi-threaded-message-loop",
            "--no-sandbox",

            "--disable-back-forward-cache",

            "--disable-web-security",
            "--disable-geolocation",

            "--disable-surfaces",

            "--autoplay-policy=no-user-gesture-required",

            "--enable-media-stream",
            "--enable-accelerated-video-decode",

            "--allow-running-insecure-content",
            "--use-fake-ui-for-media-stream",
            "--enable-usermedia-screen-capture",
            "--enable-usermedia-screen-capturing",
            "--debug-plugin-loading",
            "--allow-outdated-plugins",
            "--always-authorize-plugins",
            "--enable-npapi",

            "--disable-speech-input",
        };

        public static Dictionary<string, string> CefArguments = new()
        {
            { "enable-gpu", "1" },
            { "enable-gpu-vsync", "1" },
            { "disable-gpu-compositing", "1" },
            { "disable-direct-write", "1" },
            //{ "disable-frame-rate-limit", "1" },
            { "enable-begin-frame-scheduling", "1" },
            { "disable-breakpad", "1" },
            { "disable-extensions", "1" },

            { "multi-threaded-message-loop", "1" },
            { "no-sandbox", "1" },
            { "off-screen-rendering-enabled", "1" },

            { "disable-back-forward-cache", "1" },

            { "disable-web-security", "1" },
            { "disable-geolocation", "1" },

            { "disable-surfaces", "1" },

            { "autoplay-policy", "no-user-gesture-required" },

            { "enable-media-stream", "1" },
            { "enable-accelerated-video-decode", "1" },

            { "allow-running-insecure-content", "1" },
            { "use-fake-ui-for-media-stream", "1" },
            { "enable-usermedia-screen-capture", "1" },
            { "enable-usermedia-screen-capturing", "1" },
            { "debug-plugin-loading", "1" },
            { "allow-outdated-plugins", "1" },
            { "always-authorize-plugins", "1" },
            { "enable-npapi", "1" }
        };
    }
}