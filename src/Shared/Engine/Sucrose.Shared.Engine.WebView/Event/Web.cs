using Microsoft.Web.WebView2.Core;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.Event
{
    internal static class Web
    {
        public static void WebEngineInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            SSEMI.Initialized = e.IsSuccess;

            SSEWVMI.WebEngine.Source = SSEHS.GetSource(SSEWVMI.Web);

            //SSEWVMI.WebEngine.CoreWebView2.OpenDevToolsWindow();
        }
    }
}