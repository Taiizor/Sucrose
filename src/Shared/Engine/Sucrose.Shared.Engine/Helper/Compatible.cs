using SESMIEN = Sucrose.Shared.Engine.Manage.Internal.ExecuteNormal;
using SESMIET = Sucrose.Shared.Engine.Manage.Internal.ExecuteTask;
using SSDMMG = Sucrose.Shared.Dependency.Manage.Manager.General;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSWEW = Sucrose.Shared.Watchdog.Extension.Watch;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Compatible
    {
        public static async void ExecuteTask(SESMIET Function)
        {
            try
            {
                if (SSEMI.Initialized)
                {
                    SESMIEN AdaptedFunction = new(async (Script) =>
                    {
                        try
                        {
                            await Function(Script);
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });

                    ExecuteNormal(AdaptedFunction);
                }
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }

        public static async void ExecuteNormal(SESMIEN Function)
        {
            try
            {
                if (SSEMI.Initialized)
                {
                    if (!string.IsNullOrEmpty(SSEMI.Compatible.LoopMode))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.LoopMode, SSEHD.GetLoop()));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }

                    if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemCpu))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.SystemCpu, SSEMI.CpuData));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }

                    if (!string.IsNullOrEmpty(SSEMI.Compatible.ThemeType))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.ThemeType, SSDMMG.ThemeType));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }

                    if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemBios))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.SystemBios, SSEMI.BiosData));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }

                    if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemDate))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.SystemDate, SSEMI.DateData));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }

                    if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemAudio))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.SystemAudio, SSEMI.AudioData));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }

                    if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemTheme))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.SystemTheme, SSDMMG.ThemeType));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }

                    if (!string.IsNullOrEmpty(SSEMI.Compatible.VolumeLevel))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.VolumeLevel, SSEHD.GetVolume()));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }

                    if (!string.IsNullOrEmpty(SSEMI.Compatible.ShuffleMode))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.ShuffleMode, SSEHD.GetShuffle()));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }

                    if (!string.IsNullOrEmpty(SSEMI.Compatible.StretchMode))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.StretchMode, SSEHD.GetStretch()));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }

                    if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemMemory))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.SystemMemory, SSEMI.MemoryData));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }

                    if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemBattery))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.SystemBattery, SSEMI.BatteryData));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }

                    if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemGraphic))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.SystemGraphic, SSEMI.GraphicData));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }

                    if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemNetwork))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.SystemNetwork, SSEMI.NetworkData));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }

                    if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemMotherboard))
                    {
                        try
                        {
                            Function(string.Format(SSEMI.Compatible.SystemMotherboard, SSEMI.MotherboardData));
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    }
                }
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }

            await Task.CompletedTask;
        }
    }
}