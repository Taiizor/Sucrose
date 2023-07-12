using System.IO;
using SEAT = Skylark.Enum.AssemblyType;
using SDEWT = Sucrose.Dependency.Enum.WallpaperType;
using SHA = Skylark.Helper.Assemblies;
using SMR = Sucrose.Memory.Readonly;
using SDEET = Sucrose.Dependency.Enum.EngineType;
using SDEVET = Sucrose.Dependency.Enum.VideoEngineType;
using SDEWET = Sucrose.Dependency.Enum.WebEngineType;
using SDEUET = Sucrose.Dependency.Enum.UrlEngineType;
using SDEGET = Sucrose.Dependency.Enum.GifEngineType;
using SDEAET = Sucrose.Dependency.Enum.ApplicationEngineType;
using SDEYTET = Sucrose.Dependency.Enum.YouTubeEngineType;

namespace Sucrose.Space.Manage
{
    internal static class Internal
    {
        public static SDEGET GifEngine = SDEGET.Unknown;

        public static SDEUET UrlEngine = SDEUET.WebView;

        public static SDEWET WebEngine = SDEWET.WebView;

        public static SDEVET VideoEngine = SDEVET.WebView;

        public static SDEAET ApplicationEngine = SDEAET.App;

        public static SDEYTET YouTubeEngine = SDEYTET.WebView;

        public static string This => Path.GetDirectoryName(App);

        public static string Folder => Path.Combine(This, @"..\");

        public static string App => SHA.Assemble(SEAT.Executing).Location;

        public static string Commandog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.Commandog), SMR.Commandog);

        public static string WPFTrayIcon => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WPFTrayIcon), SMR.WPFTrayIcon);

        public static string WinFormsTrayIcon => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WinFormsTrayIcon), SMR.WinFormsTrayIcon);

        public static string WPFUserInterface => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WPFUserInterface), SMR.WPFUserInterface);

        public static string WinFormsUserInterface => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WinFormsUserInterface), SMR.WinFormsUserInterface);

        public static Dictionary<SDEET, string> EngineLive => new()
        {
            { SDEET.AppLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.AppLive), SMR.AppLive) },
            { SDEET.WebViewLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WebViewLive), SMR.WebViewLive) },
            { SDEET.CefSharpLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.CefSharpLive), SMR.CefSharpLive) },
            { SDEET.MediaElementLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.MediaElementLive), SMR.MediaElementLive) }
        };

        public static Dictionary<string, string> TextEngineLive => new()
        {
            { SMR.AppLive, EngineLive[SDEET.AppLive] },
            { SMR.WebViewLive, EngineLive[SDEET.WebViewLive] },
            { SMR.CefSharpLive, EngineLive[SDEET.CefSharpLive] },
            { SMR.MediaElementLive, EngineLive[SDEET.MediaElementLive] }
        };
    }
}