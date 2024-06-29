using System.Diagnostics;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;
using Timer = System.Timers.Timer;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Volume
    {
        public static void Start()
        {
            int Second = 5;

            Timer Volumer = new(Second * 1000);

            Volumer.Elapsed += (s, e) =>
            {
                try
                {
                    if (SMMM.VolumeActive)
                    {
                        foreach (Process Process in Process.GetProcesses().Where(Proc => !Proc.ProcessName.Contains(SMR.AppName) && !SSEMI.Processes.ToList().Any(Id => Id == Proc.Id) && !SSEMI.Applications.ToList().Any(App => App.Process.Id == Proc.Id)))
                        {
                            bool? Muted;
                            float? Volume;
                            bool? AudioActive;

                            try
                            {
                                AudioActive = SWEACAM.IsApplicationAudioActive(Process.Id);

                                if (AudioActive != null && (bool)AudioActive)
                                {
                                    Muted = SWEACAM.GetApplicationMute(Process.Id);

                                    if (Muted != null && !(bool)Muted)
                                    {
                                        Volume = SWEACAM.GetApplicationVolume(Process.Id);

                                        if (Volume != null && Volume > 0)
                                        {
                                            SSEMI.PauseVolume = true;
                                            return;
                                        }
                                    }
                                }
                            }
                            catch
                            {
                                try
                                {
                                    AudioActive = SWEVPCAM.IsApplicationAudioActive(Process.Id);

                                    if (AudioActive != null && (bool)AudioActive)
                                    {
                                        Muted = SWEVPCAM.GetApplicationMute(Process.Id);

                                        if (Muted != null && !(bool)Muted)
                                        {
                                            Volume = SWEVPCAM.GetApplicationVolume(Process.Id);

                                            if (Volume != null && Volume > 0)
                                            {
                                                SSEMI.PauseVolume = true;
                                                return;
                                            }
                                        }
                                    }
                                }
                                catch { }
                            }
                        }

                        SSEMI.PauseVolume = false;
                    }
                    else
                    {
                        SSEMI.PauseVolume = false;
                    }
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