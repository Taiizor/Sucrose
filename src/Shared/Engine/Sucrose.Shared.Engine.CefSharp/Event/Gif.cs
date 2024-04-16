using CefSharp;
using System.Windows;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSECSHG = Sucrose.Shared.Engine.CefSharp.Helper.Gif;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.Event
{
    internal static class Gif
    {
        public static void CefEngineInitializedChanged(object sender, DependencyPropertyChangedEventArgs e)
        {
            if (SMMM.DeveloperMode)
            {
                SSECSMI.CefEngine.ShowDevTools();
            }

            SSEMI.Initialized = SSECSMI.CefEngine.IsBrowserInitialized;
        }

        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            SSECSHG.SetStretch(SSEHD.GetStretch());
            SSECSHG.SetLoop(SSEHD.GetLoop());
        }

        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            Uri Gif = SSEHS.GetSource(SSECSMI.Gif);

            string Path = SSEHS.GetGifContentPath();

            SSEHS.WriteGifContent(Path, Gif);

            SSECSMI.CefEngine.Address = SSEHS.GetSource(Path).ToString();
        }
    }
}