using CefSharp;
using System.Windows;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSECSHV = Sucrose.Shared.Engine.CefSharp.Helper.Video;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.Event
{
    internal static class Video
    {
        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            //
        }

        public static void CefEngineInitializedChanged(object sender, EventArgs e)
        {
            if (SMMM.DeveloperMode)
            {
                SSECSMI.CefEngine.ShowDevTools();
            }

            SSEMI.Initialized = SSECSMI.CefEngine.IsBrowserInitialized;
        }

        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            SSECSHV.Load();
        }
    }
}