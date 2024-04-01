using CefSharp;
using SHS = Skylark.Helper.Skymath;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSECSHM = Sucrose.Shared.Engine.CefSharp.Helper.Management;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHC = Sucrose.Shared.Engine.Helper.Compatible;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSMI = Sucrose.Signal.Manage.Internal;
using SSSSBSS = Sucrose.Shared.Signal.Services.BackgroundogSignalService;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class Web
    {
        public static void StartCompatible()
        {
            if (SSEMI.Compatible.State && !SSEMI.CompatibleTimer.IsEnabled)
            {
                SSEMI.CompatibleTimer.Interval = TimeSpan.FromMilliseconds(SHS.Clamp(SSEMI.Compatible.TriggerTime, 1, int.MaxValue));
                SMMI.BackgroundogSettingManager.SetSetting(SMC.AudioRequired, !string.IsNullOrEmpty(SSEMI.Compatible.SystemAudio));
                SSEMI.CompatibleTimer.Tick += (s, e) => SSEHC.ExecuteNormal(SSECSMI.CefEngine.ExecuteScriptAsync);
                SMMI.BackgroundogSettingManager.SetSetting(SMC.SignalRequired, true);
                SSMI.BackgroundogManager.StartChannel(SSSSBSS.Handler);
                SSEMI.CompatibleTimer.Start();
            }
        }

        public static async void SetVolume(int Volume)
        {
            if (SSECSMI.Processes.Any())
            {
                foreach (int Process in SSECSMI.Processes.ToList())
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

            if (SSECSMI.Try < 3)
            {
                await Task.Run(() =>
                {
                    SSECSMI.Try++;
                    SSECSHM.SetProcesses();
                });
            }
        }
    }
}