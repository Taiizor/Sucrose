using SHS = Skylark.Helper.Skymath;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSEHC = Sucrose.Shared.Engine.Helper.Compatible;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVHM = Sucrose.Shared.Engine.WebView.Helper.Management;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SSMI = Sucrose.Signal.Manage.Internal;
using SSSSBSS = Sucrose.Shared.Signal.Services.BackgroundogSignalService;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;

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
                SMMI.BackgroundogSettingManager.SetSetting(SMC.SignalRequired, true);
                SSMI.BackgroundogManager.StartChannel(SSSSBSS.Handler);
                SSEMI.CompatibleTimer.Start();
            }
        }

        public static async void SetVolume(int Volume)
        {
            if (SSEWVMI.Processes.Any())
            {
                foreach (int Process in SSEWVMI.Processes.ToList())
                {
                    try
                    {
                        SWEVPCAM.SetApplicationVolume(Process, Volume);
                    }
                    catch
                    {
                        try
                        {
                            SWEACAM.SetApplicationVolume(Process, Volume);
                        }
                        catch { }
                    }
                }
            }

            if (SSEWVMI.Try < 3)
            {
                await Task.Run(() =>
                {
                    SSEWVMI.Try++;
                    SSEWVHM.SetProcesses();
                });
            }
        }
    }
}