﻿using System.IO;
using SEAT = Skylark.Enum.AssemblyType;
using SHA = Skylark.Helper.Assemblies;
using SMR = Sucrose.Memory.Readonly;
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

        public static string Portal => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.Portal), SMR.Portal);

        public static string Update => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.Update), SMR.Update);

        public static string Property => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.Property), SMR.Property);

        public static string Launcher => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.Launcher), SMR.Launcher);

        public static string Watchdog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.Watchdog), SMR.Watchdog);

        public static string Commandog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.Commandog), SMR.Commandog);

        public static string Reportdog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.Reportdog), SMR.Reportdog);

        public static string Backgroundog => Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.Backgroundog), SMR.Backgroundog);

        public static Dictionary<SSDEET, string> EngineLive => new()
        {
            { SSDEET.AuroraLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.AuroraLive), SMR.AuroraLive) },
            { SSDEET.NebulaLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.NebulaLive), SMR.NebulaLive) },
            { SSDEET.VexanaLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.VexanaLive), SMR.VexanaLive) },
            { SSDEET.XavierLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.XavierLive), SMR.XavierLive) },
            { SSDEET.WebViewLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.WebViewLive), SMR.WebViewLive) },
            { SSDEET.CefSharpLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.CefSharpLive), SMR.CefSharpLive) },
            { SSDEET.MpvPlayerLive, Path.Combine(Folder, Path.GetFileNameWithoutExtension(SMR.MpvPlayerLive), SMR.MpvPlayerLive) }
        };

        public static Dictionary<string, string> TextEngineLive => new()
        {
            { SMR.AuroraLive, EngineLive[SSDEET.AuroraLive] },
            { SMR.NebulaLive, EngineLive[SSDEET.NebulaLive] },
            { SMR.VexanaLive, EngineLive[SSDEET.VexanaLive] },
            { SMR.XavierLive, EngineLive[SSDEET.XavierLive] },
            { SMR.WebViewLive, EngineLive[SSDEET.WebViewLive] },
            { SMR.CefSharpLive, EngineLive[SSDEET.CefSharpLive] },
            { SMR.MpvPlayerLive, EngineLive[SSDEET.MpvPlayerLive] }
        };
    }
}