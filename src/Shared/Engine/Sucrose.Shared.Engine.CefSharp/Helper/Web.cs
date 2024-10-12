﻿using CefSharp;
using SMMCB = Sucrose.Memory.Manage.Constant.Backgroundog;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPMI = Sucrose.Pipe.Manage.Internal;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommunicationType;
using SSDMMB = Sucrose.Shared.Dependency.Manage.Manager.Backgroundog;
using SSECSHM = Sucrose.Shared.Engine.CefSharp.Helper.Management;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHC = Sucrose.Shared.Engine.Helper.Compatible;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSMI = Sucrose.Signal.Manage.Internal;
using SSPSBSS = Sucrose.Shared.Pipe.Services.BackgroundogPipeService;
using SSSSBSS = Sucrose.Shared.Signal.Services.BackgroundogSignalService;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class Web
    {
        public static void Play()
        {
            if (!SSECSMI.State)
            {
                SSECSMI.State = true;

                //SSECSMI.CefEngine.Address = SSEHS.GetSource(SSECSMI.Web).ToString();

                if (SSEMI.Processes.Any())
                {
                    foreach (int Process in SSEMI.Processes.ToList())
                    {
                        _ = SWNM.DebugActiveProcessStop((uint)Process);
                    }
                }

                //if (SSEMI.IntermediateD3DWindow > 0)
                //{
                //    _ = SWNM.DebugActiveProcessStop((uint)SSEMI.IntermediateD3DWindow);
                //}
            }
        }

        public static void Pause()
        {
            if (SSECSMI.State)
            {
                SSECSMI.State = false;

                //string Path = SSEHS.GetImageContentPath();

                //SSEHS.WriteImageContent(Path, SSECSES.Capture());

                //SSECSMI.CefEngine.Address = SSEHS.GetSource(Path).ToString();

                if (SSEMI.Processes.Any())
                {
                    foreach (int Process in SSEMI.Processes.ToList())
                    {
                        _ = SWNM.DebugActiveProcess((uint)Process);
                    }
                }

                //if (SSEMI.IntermediateD3DWindow > 0)
                //{
                //    _ = SWNM.DebugActiveProcess((uint)SSEMI.IntermediateD3DWindow);
                //}
            }
        }

        public static void StartCompatible()
        {
            if (SSEMI.Compatible.State)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.AudioRequired, !string.IsNullOrEmpty(SSEMI.Compatible.SystemAudio));

                switch (SSDMMB.CommunicationType)
                {
                    case SSDECT.Pipe:
                        SMMI.BackgroundogSettingManager.SetSetting(SMMCB.PipeRequired, true);

                        _ = Task.Run(() =>
                        {
                            SPMI.BackgroundogManager.MessageReceived += async (s, e) =>
                            {
                                if (SSECSMI.State)
                                {
                                    SSPSBSS.Handler(e);

                                    await System.Windows.Application.Current.Dispatcher.InvokeAsync(async () =>
                                    {
                                        try
                                        {
                                            if (!SSECSMI.CefEngine.IsDisposed && SSECSMI.CefEngine.IsInitialized && SSECSMI.CefEngine.CanExecuteJavascriptInMainFrame)
                                            {
                                                SSEHC.ExecuteNormal(SSECSMI.CefEngine.ExecuteScriptAsync);
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
                        SMMI.BackgroundogSettingManager.SetSetting(SMMCB.SignalRequired, true);

                        SSMI.BackgroundogManager.StartChannel(async (s, e) =>
                        {
                            if (SSECSMI.State)
                            {
                                SSSSBSS.Handler(s, e);

                                await System.Windows.Application.Current.Dispatcher.InvokeAsync(async () =>
                                {
                                    try
                                    {
                                        if (!SSECSMI.CefEngine.IsDisposed && SSECSMI.CefEngine.IsInitialized && SSECSMI.CefEngine.CanExecuteJavascriptInMainFrame)
                                        {
                                            SSEHC.ExecuteNormal(SSECSMI.CefEngine.ExecuteScriptAsync);
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