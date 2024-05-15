using Microsoft.Web.WebView2.Core;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVHM = Sucrose.Shared.Engine.WebView.Helper.Management;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SSEWVEI = Sucrose.Shared.Engine.WebView.Extension.Interaction;
using SEIT = Skylark.Enum.InputType;

namespace Sucrose.Shared.Engine.WebView.Event
{
    internal static class Url
    {
        public static void WebEngineContentLoading(object sender, CoreWebView2ContentLoadingEventArgs e)
        {
            SSEWVHM.SetProcesses();
        }

        public static void WebEngineDOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            SSEWVHM.SetProcesses();

            SSEMI.Initialized = true;

            if (SMMM.InputType != SEIT.Close)
            {
                SSEWVEI.Register();
            }
        }

        public static void WebEngineInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            SSEWVMI.WebEngine.CoreWebView2.Settings.UserAgent = SMMM.UserAgent;

            SSEWVMI.WebEngine.Source = new(SSEWVMI.Url);

            SSEWVMI.WebEngine.CoreWebView2.ContentLoading += WebEngineContentLoading;
            SSEWVMI.WebEngine.CoreWebView2.DOMContentLoaded += WebEngineDOMContentLoaded;

            if (SMMM.DeveloperMode)
            {
                SSEWVMI.WebEngine.CoreWebView2.OpenDevToolsWindow();
            }
        }
    }
}