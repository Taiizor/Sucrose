using SHS = Skylark.Helper.Skymath;
using SSEHC = Sucrose.Shared.Engine.Helper.Compatible;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Web
    {
        public static void StartCompatible()
        {
            if (SSEMI.Compatible.State && !SSEMI.CompatibleTimer.IsEnabled)
            {
                SSEMI.CompatibleTimer.Interval = TimeSpan.FromMilliseconds(SHS.Clamp(SSEMI.Compatible.TriggerTime, 1, int.MaxValue));
                SSEMI.CompatibleTimer.Tick += (s, e) => SSEHC.ExecuteTask(SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync);
                SSEMI.CompatibleTimer.Start();
            }
        }
    }
}