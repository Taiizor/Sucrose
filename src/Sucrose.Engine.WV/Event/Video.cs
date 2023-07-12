using Microsoft.Web.WebView2.Core;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SEWVHV = Sucrose.Engine.WV.Helper.Video;
using SEWVMI = Sucrose.Engine.WV.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SDEST = Sucrose.Dependency.Enum.StretchType;

namespace Sucrose.Engine.WV.Event
{
    internal static class Video
    {
        public static void WebEngineDOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            SEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].requestFullscreen();");
            SEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].controls = false;");
            SEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].loop = true;");

            SEWVHV.SetStretch(SMMI.EngineSettingManager.GetSettingStable(SMC.StretchType, SDEST.Fill));
            SEWVHV.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));
        }

        public static void WebEngineInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            Uri Video = SESHS.GetSource(SEWVMI.Video);

            if (SESHS.GetExtension(Video))
            {
                SEWVMI.WebEngine.Source = Video;
            }
            else
            {
                string Path = SESHS.GetVideoContentPath();

                SESHS.WriteVideoContent(Path, Video);

                SEWVMI.WebEngine.Source = SESHS.GetSource(Path);
            }

            SEWVMI.WebEngine.CoreWebView2.DOMContentLoaded += WebEngineDOMContentLoaded;

            //SEWVMI.WebEngine.CoreWebView2.OpenDevToolsWindow();
        }
    }
}