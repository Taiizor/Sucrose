using System.IO;
using SEAT = Skylark.Enum.AssemblyType;
using SEWT = Skylark.Enum.WallpaperType;
using SHA = Skylark.Helper.Assemblies;
using SMC = Sucrose.Memory.Constant;
using SMR = Sucrose.Memory.Readonly;
using SSEET = Sucrose.Space.Enum.EngineType;

namespace Sucrose.Space.Manage
{
    internal static class Internal
    {
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

        public static Dictionary<SEWT, SSEET> WallpaperLive => new()
        {
            { SEWT.Web, SSEET.WebViewLive },
            { SEWT.Url, SSEET.WebViewLive },
            { SEWT.Video, SSEET.WebViewLive },
            { SEWT.YouTube, SSEET.WebViewLive },
            { SEWT.Application, SSEET.AppLive },
            { SEWT.Gif, SSEET.MediaElementLive }
        };

        public static Dictionary<string, SSEET> TextWallpaperLive => new()
        {
            { SMC.AApp, SSEET.AppLive },
            { SMC.WApp, SSEET.WebViewLive },
            { SMC.UApp, SSEET.WebViewLive },
            { SMC.YApp, SSEET.WebViewLive },
            { SMC.VApp, SSEET.WebViewLive },
            { SMC.GApp, SSEET.MediaElementLive }
        };
    }
}