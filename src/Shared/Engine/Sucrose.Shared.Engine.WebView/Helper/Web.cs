using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPMI = Sucrose.Pipe.Manage.Internal;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommunicationType;
using SSEHC = Sucrose.Shared.Engine.Helper.Compatible;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEMM = Sucrose.Shared.Engine.Manage.Manager;
using SSEWVES = Sucrose.Shared.Engine.WebView.Extension.Screenshot;
using SSEWVHM = Sucrose.Shared.Engine.WebView.Helper.Management;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SSMI = Sucrose.Signal.Manage.Internal;
using SSPSBSS = Sucrose.Shared.Pipe.Services.BackgroundogPipeService;
using SSSSBSS = Sucrose.Shared.Signal.Services.BackgroundogSignalService;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Web
    {
        public static void Play()
        {
            if (!SSEWVMI.State)
            {
                SSEWVMI.State = true;

                SSEWVMI.WebEngine.Source = SSEHS.GetSource(SSEWVMI.Web);
            }
        }

        public static async void Pause()
        {
            if (SSEWVMI.State)
            {
                SSEWVMI.State = false;

                string Path = SSEHS.GetImageContentPath();

                SSEHS.WriteImageContent(Path, await SSEWVES.Capture());

                SSEWVMI.WebEngine.Source = SSEHS.GetSource(Path);
            }
        }

        public static void StartCompatible()
        {
            if (SSEMI.Compatible.State)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.AudioRequired, !string.IsNullOrEmpty(SSEMI.Compatible.SystemAudio));

                switch (SSEMM.CommunicationType)
                {
                    case SSDECT.Pipe:
                        SMMI.BackgroundogSettingManager.SetSetting(SMC.PipeRequired, true);

                        _ = Task.Run(() =>
                        {
                            SPMI.BackgroundogManager.MessageReceived += async (s, e) =>
                            {
                                if (SSEWVMI.State)
                                {
                                    SSPSBSS.Handler(e);

                                    await System.Windows.Application.Current.Dispatcher.InvokeAsync(() =>
                                    {
                                        SSEHC.ExecuteTask(SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync);
                                    });
                                }
                            };

                            SPMI.BackgroundogManager.StartServer();
                        });
                        break;
                    case SSDECT.Signal:
                        SMMI.BackgroundogSettingManager.SetSetting(SMC.SignalRequired, true);

                        SSMI.BackgroundogManager.StartChannel(async (s, e) =>
                        {
                            if (SSEWVMI.State)
                            {
                                SSSSBSS.Handler(s, e);

                                await System.Windows.Application.Current.Dispatcher.InvokeAsync(() =>
                                {
                                    SSEHC.ExecuteTask(SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync);
                                });
                            }
                        });
                        break;
                    default:
                        break;
                }
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