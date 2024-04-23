using Microsoft.Web.WebView2.Core;
using System.IO;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEHP = Sucrose.Shared.Engine.Helper.Properties;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVHW = Sucrose.Shared.Engine.WebView.Helper.Web;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SSTHP = Sucrose.Shared.Theme.Helper.Properties;

namespace Sucrose.Shared.Engine.WebView.Event
{
    internal static class Web
    {
        private static async void PropertiesWatcher(object sender, FileSystemEventArgs e)
        {
            await System.Windows.Application.Current.Dispatcher.InvokeAsync(() =>
            {
                try
                {
                    SSEMI.Properties = SSTHP.ReadJson(e.FullPath);

                    SSEHP.ExecuteTask(SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync);
                }
                catch { }
            });
        }

        public static void WebEngineDOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            SSEMI.Initialized = true;

            if (SSEMI.Properties.State)
            {
                SSEHP.ExecuteTask(SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync);

                SSEHP.ChangedEventHandler += PropertiesWatcher;

                SSEHP.StartWatcher();
            }
        }

        public static void WebEngineInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            SSEWVHW.StartCompatible();

            SSEWVMI.WebEngine.CoreWebView2.Settings.UserAgent = SMMM.UserAgent;

            SSEWVMI.WebEngine.Source = SSEHS.GetSource(SSEWVMI.Web);

            SSEWVMI.WebEngine.CoreWebView2.DOMContentLoaded += WebEngineDOMContentLoaded;

            if (SMMM.DeveloperMode)
            {
                SSEWVMI.WebEngine.CoreWebView2.OpenDevToolsWindow();
            }
        }
    }
}