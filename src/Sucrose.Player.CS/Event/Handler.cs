using CefSharp;
using Sucrose.Player.CS.Manage;
using System.Windows;

namespace Sucrose.Player.CS.Event
{
    internal static class Handler
    {
        public static void CefPlayerFrameLoadEnd(object? sender, FrameLoadEndEventArgs e)
        {
            //Internal.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].webkitRequestFullscreen();");
            //Internal.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].requestFullscreen();");
            Internal.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].controls = false;");
            Internal.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].loop = true;");

            Internal.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style = \"position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 9999;\";");
        }

        public static void CefPlayerLoaded(object sender, RoutedEventArgs e)
        {
            Internal.CefPlayer.ShowDevTools();
        }
    }
}