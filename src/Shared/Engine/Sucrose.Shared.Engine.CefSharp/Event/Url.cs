using CefSharp;
using System.Windows;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSECSHM = Sucrose.Shared.Engine.CefSharp.Helper.Management;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.Event
{
    internal static class Url
    {
        public static void CefEngineInitializedChanged(object sender, DependencyPropertyChangedEventArgs e)
        {
            SSEMI.Initialized = SSECSMI.CefEngine.IsBrowserInitialized;
        }

        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            SSECSHM.SetProcesses();
        }

        public static void CefEngineInitialized(object sender, EventArgs e)
        {
            SSECSHM.SetProcesses();
        }

        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            SSECSMI.CefEngine.Address = SSECSMI.Url;

            if (SMMM.DeveloperMode)
            {
                SSECSMI.CefEngine.ShowDevTools();
            }
        }
    }
}