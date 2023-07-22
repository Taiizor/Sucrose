using System.IO;
using SDEAET = Sucrose.Dependency.Enum.ApplicationEngineType;
using SDEET = Sucrose.Dependency.Enum.EngineType;
using SDEGET = Sucrose.Dependency.Enum.GifEngineType;
using SDEUET = Sucrose.Dependency.Enum.UrlEngineType;
using SDEVET = Sucrose.Dependency.Enum.VideoEngineType;
using SDEWET = Sucrose.Dependency.Enum.WebEngineType;
using SDEYTET = Sucrose.Dependency.Enum.YouTubeEngineType;
using SEAT = Skylark.Enum.AssemblyType;
using SHA = Skylark.Helper.Assemblies;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Space.Manage
{
    internal static class Internal
    {
        public static SDEGET GifEngine = SDEGET.Vexana;

        public static SDEUET UrlEngine = SDEUET.WebView;

        public static SDEWET WebEngine = SDEWET.WebView;

        public static SDEVET VideoEngine = SDEVET.Nebula;

        public static SDEYTET YouTubeEngine = SDEYTET.WebView;

        public static SDEAET ApplicationEngine = SDEAET.Aurora;

        public static string This => Path.GetDirectoryName(App);

        public static string Folder => Path.Combine(This, @"..\");

        public static string App => SHA.Assemble(SEAT.Executing).Location;

        public static string Portal => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.Portal), SMR.Portal);

        public static string Launcher => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.Launcher), SMR.Launcher);

        public static string Commandog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.Commandog), SMR.Commandog);

        public static Dictionary<SDEET, string> EngineLive => new()
        {
            { SDEET.AuroraLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.AuroraLive), SMR.AuroraLive) },
            { SDEET.NebulaLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.NebulaLive), SMR.NebulaLive) },
            { SDEET.VexanaLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.VexanaLive), SMR.VexanaLive) },
            { SDEET.WebViewLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WebViewLive), SMR.WebViewLive) },
            { SDEET.CefSharpLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.CefSharpLive), SMR.CefSharpLive) }
        };

        public static Dictionary<string, string> TextEngineLive => new()
        {
            { SMR.AuroraLive, EngineLive[SDEET.AuroraLive] },
            { SMR.NebulaLive, EngineLive[SDEET.NebulaLive] },
            { SMR.VexanaLive, EngineLive[SDEET.VexanaLive] },
            { SMR.WebViewLive, EngineLive[SDEET.WebViewLive] },
            { SMR.CefSharpLive, EngineLive[SDEET.CefSharpLive] }
        };
    }
}