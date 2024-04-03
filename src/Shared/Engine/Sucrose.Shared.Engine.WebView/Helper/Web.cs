using SMC = Sucrose.Memory.Constant;
using SPMI = Sucrose.Pipe.Manage.Internal;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSEHC = Sucrose.Shared.Engine.Helper.Compatible;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVHM = Sucrose.Shared.Engine.WebView.Helper.Management;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SSPSBSS = Sucrose.Shared.Pipe.Services.BackgroundogPipeService;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Web
    {
        public static void StartCompatible()
        {
            if (SSEMI.Compatible.State)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.AudioRequired, !string.IsNullOrEmpty(SSEMI.Compatible.SystemAudio));
                SMMI.BackgroundogSettingManager.SetSetting(SMC.PipeRequired, true);

                _ = Task.Run(() =>
                {
                    SPMI.BackgroundogManager.MessageReceived += async (s, e) =>
                    {
                        SSPSBSS.Handler(e);

                        await System.Windows.Application.Current.Dispatcher.InvokeAsync(() =>
                        {
                            SSEHC.ExecuteTask(SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync);
                        });
                    };

                    SPMI.BackgroundogManager.StartServer();
                });
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