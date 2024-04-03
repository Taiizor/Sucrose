using CefSharp;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPMI = Sucrose.Pipe.Manage.Internal;
using SSECSHM = Sucrose.Shared.Engine.CefSharp.Helper.Management;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHC = Sucrose.Shared.Engine.Helper.Compatible;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEMM = Sucrose.Shared.Engine.Manage.Manager;
using SSPSBSS = Sucrose.Shared.Pipe.Services.BackgroundogPipeService;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommunicationType;
using SSMI = Sucrose.Signal.Manage.Internal;
using SSSSBSS = Sucrose.Shared.Signal.Services.BackgroundogSignalService;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class Web
    {
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
                                SSPSBSS.Handler(e);

                                await System.Windows.Application.Current.Dispatcher.InvokeAsync(() =>
                                {
                                    SSEHC.ExecuteNormal(SSECSMI.CefEngine.ExecuteScriptAsync);
                                });
                            };

                            SPMI.BackgroundogManager.StartServer();
                        });
                        break;
                    case SSDECT.Signal:
                        SMMI.BackgroundogSettingManager.SetSetting(SMC.SignalRequired, true);

                        SSMI.BackgroundogManager.StartChannel(async (s, e) =>
                        {
                            SSSSBSS.Handler(s, e);

                            await System.Windows.Application.Current.Dispatcher.InvokeAsync(() =>
                            {
                                SSEHC.ExecuteNormal(SSECSMI.CefEngine.ExecuteScriptAsync);
                            });
                        });
                        break;
                    default:
                        break;
                }
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