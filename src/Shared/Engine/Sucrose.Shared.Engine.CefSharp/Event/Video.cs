using CefSharp;
using System.Windows;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SSECSHV = Sucrose.Shared.Engine.CefSharp.Helper.Video;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.Event
{
    internal static class Video
    {
        public static void CefEngineInitializedChanged(object sender, DependencyPropertyChangedEventArgs e)
        {
            SESMI.Initialized = SSECSMI.CefEngine.IsBrowserInitialized;
        }

        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            //SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].webkitRequestFullscreen();");
            //SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].requestFullscreen();");
            SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].controls = false;");
            SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].loop = true;");

            SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style = \"position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 9999;\";");

            SSECSHV.SetStretch(SESHD.GetStretch());
            SSECSHV.SetVolume(SESHD.GetVolume());
        }

        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            //SSECSMI.CefEngine.ShowDevTools();
        }
    }
}