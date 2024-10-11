using System.IO;
using SEAT = Skylark.Enum.AssemblyType;
using SHA = Skylark.Helper.Assemblies;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;
using SSDEAET = Sucrose.Shared.Dependency.Enum.ApplicationEngineType;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSDEGET = Sucrose.Shared.Dependency.Enum.GifEngineType;
using SSDEUET = Sucrose.Shared.Dependency.Enum.UrlEngineType;
using SSDEVET = Sucrose.Shared.Dependency.Enum.VideoEngineType;
using SSDEWET = Sucrose.Shared.Dependency.Enum.WebEngineType;
using SSDEYTET = Sucrose.Shared.Dependency.Enum.YouTubeEngineType;

namespace Sucrose.Shared.Space.Manage
{
    internal static class Internal
    {
        public static SSDEGET GifEngine = SSDEGET.Xavier;

        public static SSDEUET UrlEngine = SSDEUET.WebView;

        public static SSDEWET WebEngine = SSDEWET.WebView;

        public static SSDEVET VideoEngine = SSDEVET.Nebula;

        public static SSDEYTET YouTubeEngine = SSDEYTET.WebView;

        public static SSDEAET ApplicationEngine = SSDEAET.Aurora;

        public static int THREAD_SUSPEND_RESUME => 0x0002;

        public static string This => Path.GetDirectoryName(App);

        public static string App => SHA.Assemble(SEAT.Executing).Location;

        public static string Folder => Directory.GetParent(This).FullName;

        public static string Portal => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Portal), SMMRA.Portal);

        public static string Update => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Update), SMMRA.Update);

        public static string Property => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Property), SMMRA.Property);

        public static string Launcher => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Launcher), SMMRA.Launcher);

        public static string Watchdog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Watchdog), SMMRA.Watchdog);

        public static string Commandog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Commandog), SMMRA.Commandog);

        public static string Reportdog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Reportdog), SMMRA.Reportdog);

        public static string Backgroundog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Backgroundog), SMMRA.Backgroundog);

        public static Dictionary<SSDEET, string> EngineLive => new()
        {
            { SSDEET.AuroraLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.AuroraLive), SMMRA.AuroraLive) },
            { SSDEET.NebulaLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.NebulaLive), SMMRA.NebulaLive) },
            { SSDEET.VexanaLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.VexanaLive), SMMRA.VexanaLive) },
            { SSDEET.XavierLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.XavierLive), SMMRA.XavierLive) },
            { SSDEET.WebViewLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.WebViewLive), SMMRA.WebViewLive) },
            { SSDEET.CefSharpLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.CefSharpLive), SMMRA.CefSharpLive) },
            { SSDEET.MpvPlayerLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.MpvPlayerLive), SMMRA.MpvPlayerLive) }
        };

        public static Dictionary<string, string> TextEngineLive => new()
        {
            { SMMRA.AuroraLive, EngineLive[SSDEET.AuroraLive] },
            { SMMRA.NebulaLive, EngineLive[SSDEET.NebulaLive] },
            { SMMRA.VexanaLive, EngineLive[SSDEET.VexanaLive] },
            { SMMRA.XavierLive, EngineLive[SSDEET.XavierLive] },
            { SMMRA.WebViewLive, EngineLive[SSDEET.WebViewLive] },
            { SMMRA.CefSharpLive, EngineLive[SSDEET.CefSharpLive] },
            { SMMRA.MpvPlayerLive, EngineLive[SSDEET.MpvPlayerLive] }
        };
    }
}