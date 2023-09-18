using Microsoft.Web.WebView2.Core;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEHP = Sucrose.Shared.Engine.Helper.Properties;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVHW = Sucrose.Shared.Engine.WebView.Helper.Web;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.Event
{
    internal static class Web
    {
        public static void WebEngineDOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            if (SSEMI.Properties.State)
            {
                SSEHP.ExecuteTask(SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync);
            }
        }

        public static void WebEngineInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            SSEWVHW.StartCompatible();

            SSEMI.Initialized = e.IsSuccess;

            SSEWVMI.WebEngine.CoreWebView2.Settings.UserAgent = SMMM.UserAgent;

            SSEWVMI.WebEngine.Source = SSEHS.GetSource(SSEWVMI.Web);

            SSEWVMI.WebEngine.CoreWebView2.DOMContentLoaded += WebEngineDOMContentLoaded;

            SSEWVMI.WebEngine.CoreWebView2.OpenDevToolsWindow();
        }
    }
}