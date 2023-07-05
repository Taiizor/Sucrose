using Microsoft.Web.WebView2.Core;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPSHS = Sucrose.Player.Shared.Helper.Source;
using SPWVHWVH = Sucrose.Player.WV.Helper.WebViewHelper;
using SPWVMI = Sucrose.Player.WV.Manage.Internal;
using SSEST = Sucrose.Space.Enum.StretchType;

namespace Sucrose.Player.WV.Event
{
    internal static class Handler
    {
        public static void EdgePlayerDOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            SPWVMI.EdgePlayer.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].requestFullscreen();");
            SPWVMI.EdgePlayer.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].controls = false;");
            SPWVMI.EdgePlayer.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].loop = true;");

            SPWVHWVH.SetStretch(SMMI.EngineSettingManager.GetSettingStable(SMC.StretchType, SSEST.Fill));
            SPWVHWVH.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));
        }

        public static void EdgePlayerInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            Uri Video = SPSHS.GetSource(SMMI.EngineSettingManager.GetSetting(SMC.Video, @""));

            if (SPSHS.GetExtension(Video))
            {
                SPWVMI.EdgePlayer.Source = Video;
            }
            else
            {
                string Path = SPSHS.GetContentPath();

                SPSHS.WriteContent(Path, Video);

                SPWVMI.EdgePlayer.Source = SPSHS.GetSource(Path);
            }

            SPWVMI.EdgePlayer.CoreWebView2.DOMContentLoaded += EdgePlayerDOMContentLoaded;

            //SPWVMI.EdgePlayer.CoreWebView2.OpenDevToolsWindow();
        }
    }
}