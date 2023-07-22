using Microsoft.Web.WebView2.Core;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.Event
{
    internal static class Web
    {
        public static void WebEngineInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            SESMI.Initialized = e.IsSuccess;

            SSEWVMI.WebEngine.Source = SESHS.GetSource(SSEWVMI.Web);

            //SSEWVMI.WebEngine.CoreWebView2.OpenDevToolsWindow();
        }
    }
}