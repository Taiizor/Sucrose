using CefSharp;
using SHS = Skylark.Helper.Skymath;
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
                SSEMI.CompatibleTimer.Interval = TimeSpan.FromMilliseconds(SHS.Clamp(SSEMI.Compatible.TriggerTime, 1, int.MaxValue));
                SSEMI.CompatibleTimer.Tick += (s, e) => SSEHC.ExecuteNormal(SSECSMI.CefEngine.ExecuteScriptAsync);
                SSEMI.CompatibleTimer.Start();
            }
        }
    }
}