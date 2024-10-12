﻿using System.Diagnostics;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;
using Timer = System.Timers.Timer;
using SMME = Sucrose.Manager.Manage.Engine;
using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SMMMCE = Sucrose.Memory.Manage.Constant.Engine;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Volume
    {
        public static void Start()
        {
            Timer Volumer = new(SMME.VolumeSensitivity * 1000);

            Volumer.Elapsed += (s, e) =>
            {
                try
                {
                    if (SMME.Volume > 0 && SMME.VolumeActive)
                    {
                        foreach (Process Process in Process.GetProcesses().Where(Proc => !Proc.ProcessName.Contains(SMR.AppName) && !SSEMI.Processes.ToList().Any(Id => Id == Proc.Id) && !SSEMI.Applications.ToList().Any(App => App.Process.Id == Proc.Id)))
                        {
                            float? Volume = 0;
                            bool? Muted = true;
                            bool? AudioActive = false;

                            try
                            {
                                AudioActive = SWEACAM.IsApplicationAudioActive(Process.Id);
                            }
                            catch
                            {
                                try
                                {
                                    AudioActive = SWEVPCAM.IsApplicationAudioActive(Process.Id);
                                }
                                catch { }
                            }

                            if (AudioActive != null && (bool)AudioActive)
                            {
                                try
                                {
                                    Muted = SWEACAM.GetApplicationMute(Process.Id);
                                }
                                catch
                                {
                                    try
                                    {
                                        Muted = SWEVPCAM.GetApplicationMute(Process.Id);
                                    }
                                    catch { }
                                }

                                if (Muted != null && !(bool)Muted)
                                {
                                    try
                                    {
                                        Volume = SWEACAM.GetApplicationVolume(Process.Id);
                                    }
                                    catch
                                    {
                                        try
                                        {
                                            Volume = SWEVPCAM.GetApplicationVolume(Process.Id);
                                        }
                                        catch { }
                                    }

                                    if (Volume is not null and > 0)
                                    {
                                        SSEMI.PauseVolume = true;
                                        return;
                                    }
                                }
                            }
                        }

                        SSEMI.PauseVolume = false;
                    }
                    else
                    {
                        SSEMI.PauseVolume = false;
                    }

                    Volumer.Interval = SMME.VolumeSensitivity * 1000;
                }
                catch
                {
                    SSEMI.PauseVolume = false;
                }
            };

            Volumer.AutoReset = true;

            Volumer.Start();
        }
    }
}