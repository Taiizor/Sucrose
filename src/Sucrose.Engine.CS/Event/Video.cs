using CefSharp;
using System.Windows;
using SECSHV = Sucrose.Engine.CS.Helper.Video;
using SECSMI = Sucrose.Engine.CS.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SDEST = Sucrose.Dependency.Enum.StretchType;

namespace Sucrose.Engine.CS.Event
{
    internal static class Video
    {
        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            //SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].webkitRequestFullscreen();");
            //SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].requestFullscreen();");
            SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].controls = false;");
            SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].loop = true;");

            SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style = \"position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 9999;\";");

            SECSHV.SetStretch(SMMI.EngineSettingManager.GetSettingStable(SMC.StretchType, SDEST.Fill));
            SECSHV.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));
        }

        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            //SECSMI.CefEngine.ShowDevTools();
        }
    }
}