using Microsoft.Web.WebView2.Core;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SEWVMI = Sucrose.Engine.WV.Manage.Internal;

namespace Sucrose.Engine.WV.Event
{
    internal static class Web
    {
        public static void WebEngineInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            SEWVMI.WebEngine.Source = SESHS.GetSource(SEWVMI.Web);

            //SEWVMI.WebEngine.CoreWebView2.OpenDevToolsWindow();
        }
    }
}