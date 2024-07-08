using Microsoft.Web.WebView2.Core;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVHV = Sucrose.Shared.Engine.WebView.Helper.Video;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.Event
{
    internal static class Video
    {
        public static void WebEngineDOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            SSEWVHV.Load();

            SSEMI.Initialized = true;
        }

        public static void WebEngineInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            SSEWVMI.WebEngine.CoreWebView2.Settings.UserAgent = SMMM.UserAgent;

            Uri Video = SSEHS.GetSource(SSEWVMI.Video);

            if (SSEHS.GetExtension(Video))
            {
                SSEWVMI.WebEngine.Source = Video;
            }
            else
            {
                string Path = SSEHS.GetVideoContentPath();

                SSEHS.WriteVideoContent(Path, Video);

                SSEWVMI.WebEngine.Source = SSEHS.GetSource(Path);
            }

            SSEWVMI.WebEngine.CoreWebView2.DOMContentLoaded += WebEngineDOMContentLoaded;

            if (SMMM.DeveloperMode)
            {
                SSEWVMI.WebEngine.CoreWebView2.OpenDevToolsWindow();
            }
        }
    }
}