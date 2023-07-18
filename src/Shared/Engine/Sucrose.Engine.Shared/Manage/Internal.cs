using SESSB = Sucrose.Engine.Shared.Setting.Browser;
using STSHP = Sucrose.Theme.Shared.Helper.Properties;

namespace Sucrose.Engine.Shared.Manage
{
    internal static class Internal
    {
        public static STSHP Properties = new();

        public static SESSB BrowserSettings = new()
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