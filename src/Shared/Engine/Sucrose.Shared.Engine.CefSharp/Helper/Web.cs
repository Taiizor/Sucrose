using CefSharp;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHC = Sucrose.Shared.Engine.Helper.Compatible;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class Web
    {
        public static void StartCompatible()
        {
            if (SSEMI.Compatible.State && !SSEMI.CompatibleTimer.IsEnabled)
            {
                SSEMI.CompatibleTimer.Tick += (s, e) => SSEHC.ExecuteNormal(SSECSMI.CefEngine.ExecuteScriptAsync);
                SSEMI.CompatibleTimer.Interval = TimeSpan.FromMilliseconds(SSEMI.Compatible.TriggerTime);
                SSEMI.CompatibleTimer.Start();
            }
        }
    }
}