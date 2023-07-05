using CefSharp;
using System.Windows;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPCSHCSH = Sucrose.Player.CS.Helper.CefSharpHelper;
using SPCSMI = Sucrose.Player.CS.Manage.Internal;
using SSEST = Sucrose.Space.Enum.StretchType;

namespace Sucrose.Player.CS.Event
{
    internal static class Handler
    {
        public static void CefPlayerFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            //SPCSMI.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].webkitRequestFullscreen();");
            //SPCSMI.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].requestFullscreen();");
            SPCSMI.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].controls = false;");
            SPCSMI.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].loop = true;");

            SPCSMI.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style = \"position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 9999;\";");

            SPCSHCSH.SetStretch(SMMI.EngineSettingManager.GetSettingStable(SMC.StretchType, SSEST.Fill));
            SPCSHCSH.SetVolume(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100));
        }

        public static void CefPlayerLoaded(object sender, RoutedEventArgs e)
        {
            //SPCSMI.CefPlayer.ShowDevTools();
        }
    }
}