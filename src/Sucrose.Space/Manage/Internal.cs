using System.IO;
using SEAT = Skylark.Enum.AssemblyType;
using SHA = Skylark.Helper.Assemblies;
using SMR = Sucrose.Memory.Readonly;
using SSEET = Sucrose.Space.Enum.EngineType;

namespace Sucrose.Space.Manage
{
    internal static class Internal
    {
        public static string Folder => Path.Combine(Path.GetDirectoryName(SHA.Assemble(SEAT.Executing).Location), @"..\");

        public static string CommandLine => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.CommandLine), SMR.CommandLine);

        public static string WPFTrayIcon => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WPFTrayIcon), SMR.WPFTrayIcon);

        public static string WinFormsTrayIcon => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WinFormsTrayIcon), SMR.WinFormsTrayIcon);

        public static string WPFUserInterface => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WPFUserInterface), SMR.WPFUserInterface);

        public static string WinFormsUserInterface => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WinFormsUserInterface), SMR.WinFormsUserInterface);

        public static Dictionary<SSEET, string> EngineLive => new()
        {
            { SSEET.EngineLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WebViewLive), SMR.EngineLive) },
            { SSEET.WebViewLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WebViewLive), SMR.WebViewLive) },
            { SSEET.CefSharpLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.CefSharpLive), SMR.CefSharpLive) },
            { SSEET.MediaElementLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.MediaElementLive), SMR.MediaElementLive) }
        };

        public static Dictionary<string, string> TextEngineLive => new()
        {
            { SMR.EngineLive, EngineLive[SSEET.EngineLive] },
            { SMR.WebViewLive, EngineLive[SSEET.WebViewLive] },
            { SMR.CefSharpLive, EngineLive[SSEET.CefSharpLive] },
            { SMR.MediaElementLive, EngineLive[SSEET.MediaElementLive] }
        };
    }
}