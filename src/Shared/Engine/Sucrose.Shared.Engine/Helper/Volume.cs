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
                            try
                            {
                                bool? isMuted = SWEACAM.GetApplicationMute(Process.Id);
                                float? volume = SWEACAM.GetApplicationVolume(Process.Id);
                                bool? isAudioActive = SWEACAM.IsApplicationAudioActive(Process.Id);

                                if (volume != null && isMuted != null && isAudioActive != null)
                                {
                                    if (volume > 0 && !(bool)isMuted && (bool)isAudioActive)
                                    {
                                        SSEMI.PauseVolume = true;
                                        return;
                                    }
                                }
                            }
                            catch
                            {
                                try
                                {
                                    bool? isMuted = SWEVPCAM.GetApplicationMute(Process.Id);
                                    float? volume = SWEVPCAM.GetApplicationVolume(Process.Id);
                                    bool? isAudioActive = SWEVPCAM.IsApplicationAudioActive(Process.Id);

                                    if (volume != null && isMuted != null && isAudioActive != null)
                                    {
                                        if (volume > 0 && !(bool)isMuted && (bool)isAudioActive)
                                        {
                                            SSEMI.PauseVolume = true;
                                            return;
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