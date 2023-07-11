using System.IO;
using SEAT = Skylark.Enum.AssemblyType;
using SSEWT = Sucrose.Space.Enum.WallpaperType;
using SHA = Skylark.Helper.Assemblies;
using SMR = Sucrose.Memory.Readonly;
using SSEET = Sucrose.Space.Enum.EngineType;
using SSEVET = Sucrose.Space.Enum.VideoEngineType;
using SSEWET = Sucrose.Space.Enum.WebEngineType;
using SSEUET = Sucrose.Space.Enum.UrlEngineType;
using SSEGET = Sucrose.Space.Enum.GifEngineType;
using SSEAET = Sucrose.Space.Enum.ApplicationEngineType;
using SSEYTET = Sucrose.Space.Enum.YouTubeEngineType;

namespace Sucrose.Space.Manage
{
    internal static class Internal
    {
        public static SSEGET GifEngine = SSEGET.Unknown;

        public static SSEUET UrlEngine = SSEUET.WebView;

        public static SSEWET WebEngine = SSEWET.WebView;

        public static SSEVET VideoEngine = SSEVET.WebView;

        public static SSEAET ApplicationEngine = SSEAET.App;

        public static SSEYTET YouTubeEngine = SSEYTET.WebView;

        public static string This => Path.GetDirectoryName(App);

        public static string Folder => Path.Combine(This, @"..\");

        public static string App => SHA.Assemble(SEAT.Executing).Location;

        public static string Commandog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.Commandog), SMR.Commandog);

        public static string WPFTrayIcon => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WPFTrayIcon), SMR.WPFTrayIcon);

        public static string WinFormsTrayIcon => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WinFormsTrayIcon), SMR.WinFormsTrayIcon);

        public static string WPFUserInterface => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WPFUserInterface), SMR.WPFUserInterface);

        public static string WinFormsUserInterface => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WinFormsUserInterface), SMR.WinFormsUserInterface);

        public static Dictionary<SSEET, string> EngineLive => new()
        {
            { SSEET.AppLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.AppLive), SMR.AppLive) },
            { SSEET.WebViewLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WebViewLive), SMR.WebViewLive) },
            { SSEET.CefSharpLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.CefSharpLive), SMR.CefSharpLive) },
            { SSEET.MediaElementLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.MediaElementLive), SMR.MediaElementLive) }
        };

        public static Dictionary<string, string> TextEngineLive => new()
        {
            { SMR.AppLive, EngineLive[SSEET.AppLive] },
            { SMR.WebViewLive, EngineLive[SSEET.WebViewLive] },
            { SMR.CefSharpLive, EngineLive[SSEET.CefSharpLive] },
            { SMR.MediaElementLive, EngineLive[SSEET.MediaElementLive] }
        };

        public static Dictionary<SSEWT, SSEET> WallpaperLive => new()
        {
            { SSEWT.Web, SSEET.WebViewLive },
            { SSEWT.Url, SSEET.WebViewLive },
            { SSEWT.Video, SSEET.WebViewLive },
            { SSEWT.YouTube, SSEET.WebViewLive },
            { SSEWT.Application, SSEET.AppLive },
            { SSEWT.Gif, SSEET.MediaElementLive }
        };
    }
}