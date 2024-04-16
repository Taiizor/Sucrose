using CefSharp;
using System.Windows;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSECSHW = Sucrose.Shared.Engine.CefSharp.Helper.Web;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHP = Sucrose.Shared.Engine.Helper.Properties;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.Event
{
    internal static class Web
    {
        public static void CefEngineInitializedChanged(object sender, DependencyPropertyChangedEventArgs e)
        {
            if (SMMM.DeveloperMode)
            {
                SSECSMI.CefEngine.ShowDevTools();
            }

            SSECSHW.StartCompatible();

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
        }
    }
}