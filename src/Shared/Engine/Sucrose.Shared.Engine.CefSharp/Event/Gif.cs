using CefSharp;
using System.Windows;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSECSHG = Sucrose.Shared.Engine.CefSharp.Helper.Gif;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SMME = Sucrose.Manager.Manage.Engine;
using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SMMMCE = Sucrose.Memory.Manage.Constant.Engine;

namespace Sucrose.Shared.Engine.CefSharp.Event
{
    internal static class Gif
    {
        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            Uri Gif = SSEHS.GetSource(SSECSMI.Gif);

            string Path = SSEHS.GetGifContentPath();

            SSEHS.WriteGifContent(Path, Gif);

            SSECSMI.CefEngine.Address = SSEHS.GetSource(Path).ToString();
        }

        public static void CefEngineInitializedChanged(object sender, EventArgs e)
        {
            if (SMME.DeveloperMode)
            {
                SSECSMI.CefEngine.ShowDevTools();
            }

            SSEMI.Initialized = SSECSMI.CefEngine.IsBrowserInitialized;
        }

        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            SSECSHG.Load();
        }
    }
}