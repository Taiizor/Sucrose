using CefSharp;
using System.Windows;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSECSHV = Sucrose.Shared.Engine.CefSharp.Helper.Video;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.Event
{
    internal static class Video
    {
        public static void CefEngineInitializedChanged(object sender, DependencyPropertyChangedEventArgs e)
        {
            SSEMI.Initialized = SSECSMI.CefEngine.IsBrowserInitialized;
        }

        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            //SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].webkitRequestFullscreen();");
            //SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].requestFullscreen();");
            SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].controls = false;");
            SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].loop = true;");
            SSECSMI.CefEngine.ExecuteScriptAsync(SSEHS.GetVideoStyle());

            SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style = \"position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 9999;\";");

            SSECSHV.SetStretch(SSEHD.GetStretch());
            SSECSHV.SetVolume(SSEHD.GetVolume());
        }

        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            if (SMMM.DeveloperMode)
            {
                SSECSMI.CefEngine.ShowDevTools();
            }
        }
    }
}