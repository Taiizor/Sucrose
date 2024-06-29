using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPMI = Sucrose.Pipe.Manage.Internal;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommunicationType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSEHC = Sucrose.Shared.Engine.Helper.Compatible;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVHM = Sucrose.Shared.Engine.WebView.Helper.Management;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SSMI = Sucrose.Signal.Manage.Internal;
using SSPSBSS = Sucrose.Shared.Pipe.Services.BackgroundogPipeService;
using SSSSBSS = Sucrose.Shared.Signal.Services.BackgroundogSignalService;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Web
    {
        public static void Play()
        {
            if (!SSEWVMI.State)
            {
                SSEWVMI.State = true;

                //SSEWVMI.WebEngine.Source = SSEHS.GetSource(SSEWVMI.Web);

                if (SSEMI.IntermediateD3DWindow > 0)
                {
                    _ = SWNM.DebugActiveProcessStop((uint)SSEMI.IntermediateD3DWindow);
                }
            }
        }

        public static void Pause()
        {
            if (SSEWVMI.State)
            {
                SSEWVMI.State = false;

                //string Path = SSEHS.GetImageContentPath();

                //SSEHS.WriteImageContent(Path, await SSEWVES.Capture());

                //SSEWVMI.WebEngine.Source = SSEHS.GetSource(Path);

                if (SSEMI.IntermediateD3DWindow > 0)
                {
                    _ = SWNM.DebugActiveProcess((uint)SSEMI.IntermediateD3DWindow);
                }
            }
        }

        public static void StartCompatible()
        {
            if (SSEMI.Compatible.State)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.AudioRequired, !string.IsNullOrEmpty(SSEMI.Compatible.SystemAudio));

                switch (SSDMM.CommunicationType)
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

                                    await System.Windows.Application.Current.Dispatcher.InvokeAsync(async () =>
                                    {
                                        try
                                        {
                                            if (SSEWVMI.WebEngine.IsInitialized)
                                            {
                                                SSEHC.ExecuteTask(SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync);
                                            }
                                        }
                                        catch (Exception Exception)
                                        {
                                            await SSWW.Watch_CatchException(Exception);
                                        }
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

                                await System.Windows.Application.Current.Dispatcher.InvokeAsync(async () =>
                                {
                                    try
                                    {
                                        if (SSEWVMI.WebEngine.IsInitialized)
                                        {
                                            SSEHC.ExecuteTask(SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync);
                                        }
                                    }
                                    catch (Exception Exception)
                                    {
                                        await SSWW.Watch_CatchException(Exception);
                                    }
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
            if (SSEMI.Processes.Any())
            {
                foreach (int Process in SSEMI.Processes.ToList())
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