using CefSharp;
using System.Windows;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEHP = Sucrose.Shared.Engine.Helper.Properties;

namespace Sucrose.Shared.Engine.CefSharp.Event
{
    internal static class Web
    {
        public static void CefEngineInitializedChanged(object sender, DependencyPropertyChangedEventArgs e)
        {
            SSEMI.Initialized = SSECSMI.CefEngine.IsBrowserInitialized;
        }

        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            if (SSEMI.Properties.State)
            {
                SSEHP.ExecuteNormal(SSECSMI.CefEngine.ExecuteScriptAsync);
            }
        }

        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            SSECSMI.CefEngine.Address = SSEHS.GetSource(SSECSMI.Web).ToString();

            //SSECSMI.CefEngine.ShowDevTools();
        }
    }
}