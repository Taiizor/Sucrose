using CefSharp;
using System.IO;
using System.Windows;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSECSHW = Sucrose.Shared.Engine.CefSharp.Helper.Web;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHP = Sucrose.Shared.Engine.Helper.Properties;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSTHP = Sucrose.Shared.Theme.Helper.Properties;

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

        private static async void PropertiesWatcher(object sender, FileSystemEventArgs e)
        {
            await System.Windows.Application.Current.Dispatcher.InvokeAsync(() =>
            {
                try
                {
                    SSEMI.Properties = SSTHP.ReadJson(e.FullPath);

                    SSEHP.ExecuteNormal(SSECSMI.CefEngine.ExecuteScriptAsync);
                }
                catch { }
            });
        }

        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            if (!string.IsNullOrEmpty(SSEMI.PropertiesFile))
            {
                SSEMI.Properties = SSTHP.ReadJson(SSEMI.PropertiesFile);
                SSEMI.Properties.State = true;
            }

            if (SSEMI.Properties.State)
            {
                SSEHP.ExecuteNormal(SSECSMI.CefEngine.ExecuteScriptAsync);

                if (SSEMI.PropertiesWatcher)
                {
                    SSEHP.CreatedEventHandler += PropertiesWatcher;
                }

                SSEHP.StartWatcher();
            }
        }

        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            SSECSMI.CefEngine.Address = SSEHS.GetSource(SSECSMI.Web).ToString();
        }
    }
}