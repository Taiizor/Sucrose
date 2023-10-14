using CefSharp;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class Gif
    {
        public static void PlayPause()
        {
            SSECSMI.CefEngine.ExecuteScriptAsync("SucrosePlayPause();");
        }

        public static void SetLoop(bool State)
        {
            SSECSMI.CefEngine.ExecuteScriptAsync($"SucroseLoopMode({State.ToString().ToLower()});");
        }

        public static void SetStretch(SSDEST Stretch)
        {
            SSECSMI.CefEngine.ExecuteScriptAsync($"SucroseStretchMode('{Stretch}');");
        }
    }
}