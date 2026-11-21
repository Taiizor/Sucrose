using System.IO;
using SEAT = Skylark.Enum.AssemblyType;
using SHA = Skylark.Helper.Assemblies;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;
using SMMRF = Sucrose.Memory.Manage.Readonly.Folder;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Process;
using SMMRPA = Sucrose.Memory.Manage.Readonly.Path;
using SSDEAET = Sucrose.Shared.Dependency.Enum.ApplicationEngineType;
using SSDEAT = Sucrose.Shared.Dependency.Enum.ArchitectureType;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSDEGET = Sucrose.Shared.Dependency.Enum.GifEngineType;
using SSDEUET = Sucrose.Shared.Dependency.Enum.UrlEngineType;
using SSDEVET = Sucrose.Shared.Dependency.Enum.VideoEngineType;
using SSDEWET = Sucrose.Shared.Dependency.Enum.WebEngineType;
using SSDEYTET = Sucrose.Shared.Dependency.Enum.YouTubeEngineType;
using SSSHE = Sucrose.Shared.Space.Helper.Extension;

namespace Sucrose.Shared.Space.Manage
{
    internal static class Internal
    {
        public static SSDEUET UrlEngine = SSDEUET.CefSharp;

        public static SSDEWET WebEngine = SSDEWET.CefSharp;

        public static SSDEGET GifEngine = SSDEGET.MpvPlayer;

        public static SSDEVET VideoEngine = SSDEVET.MpvPlayer;

        public static SSDEAET ApplicationEngine = SSDEAET.Aurora;

        public static SSDEYTET YouTubeEngine = SSDEYTET.CefSharp;

        public static int THREAD_SUSPEND_RESUME => 0x0002;

        public static string This => Path.GetDirectoryName(App);

        public static string Assets => Path.Combine(This, SMMRF.Assets);

        public static string Folder => Directory.GetParent(This).FullName;

        public static string Locales => Path.Combine(This, SMMRF.Locales);

        public static string Requirements => Path.Combine(This, SMMRF.Requirements);

        public static string CefSharpProcess => Path.Combine(This, SMMRP.CefSharpFullName);

        public static string App => SSSHE.Change(SHA.Assemble(SEAT.Entry).Location, ".exe");

        public static string Runtime => Path.Combine(Folder, $"{SMMRG.AppName}.{SMMRF.Runtime}");

        public static string Portal => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Portal), SMMRA.Portal);

        public static string Update => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Update), SMMRA.Update);

        public static string Property => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Property), SMMRA.Property);

        public static string Launcher => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Launcher), SMMRA.Launcher);

        public static string Watchdog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Watchdog), SMMRA.Watchdog);

        public static string Commandog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Commandog), SMMRA.Commandog);

        public static string Reportdog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Reportdog), SMMRA.Reportdog);

        public static string Backgroundog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.Backgroundog), SMMRA.Backgroundog);

#if X86
        public static SSDEAT Architecture => SSDEAT.x86;
#elif X64
        public static SSDEAT Architecture => SSDEAT.x64;
#elif ARM64
        public static SSDEAT Architecture => SSDEAT.ARM64;
#else
        public static SSDEAT Architecture => SSDEAT.Unknown;
#endif

        public static string Native => Path.Combine(Folder, SMMRF.Native);

        public static string LockScreenRoot => Path.Combine(SMMRPA.CommonApplicationData, SMMRG.AppName, SMMRF.LockScreen);

        public static string LockScreenCore => File.Exists(Path.Combine(Native, SMMRA.LockScreenCore))
            ? Path.Combine(Native, SMMRA.LockScreenCore)
            : Path.Combine(Native, $"{Architecture}", "Release", SMMRA.LockScreenCore);

        public static Dictionary<SSDEET, string> EngineLive => new()
        {
            [SSDEET.AuroraLive] = Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.AuroraLive), SMMRA.AuroraLive),
            [SSDEET.NebulaLive] = Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.NebulaLive), SMMRA.NebulaLive),
            [SSDEET.VexanaLive] = Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.VexanaLive), SMMRA.VexanaLive),
            [SSDEET.XavierLive] = Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.XavierLive), SMMRA.XavierLive),
            [SSDEET.WebViewLive] = Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.WebViewLive), SMMRA.WebViewLive),
            [SSDEET.CefSharpLive] = Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.CefSharpLive), SMMRA.CefSharpLive),
            [SSDEET.MpvPlayerLive] = Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.MpvPlayerLive), SMMRA.MpvPlayerLive),
            [SSDEET.VlcPlayerLive] = Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMMRA.VlcPlayerLive), SMMRA.VlcPlayerLive)
        };

        public static Dictionary<string, string> TextEngineLive => new()
        {
            [SMMRA.AuroraLive] = EngineLive[SSDEET.AuroraLive],
            [SMMRA.NebulaLive] = EngineLive[SSDEET.NebulaLive],
            [SMMRA.VexanaLive] = EngineLive[SSDEET.VexanaLive],
            [SMMRA.XavierLive] = EngineLive[SSDEET.XavierLive],
            [SMMRA.WebViewLive] = EngineLive[SSDEET.WebViewLive],
            [SMMRA.CefSharpLive] = EngineLive[SSDEET.CefSharpLive],
            [SMMRA.MpvPlayerLive] = EngineLive[SSDEET.MpvPlayerLive],
            [SMMRA.VlcPlayerLive] = EngineLive[SSDEET.VlcPlayerLive]
        };
    }
}