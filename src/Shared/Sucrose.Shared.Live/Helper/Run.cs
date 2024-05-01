using System.IO;
using SHV = Skylark.Helper.Versionly;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SWUD = Skylark.Wing.Utility.Desktop;

namespace Sucrose.Shared.Live.Helper
{
    internal static class Run
    {
        public static void Start()
        {
            if ((!SMMM.ClosePerformance && !SMMM.PausePerformance) || !SSSHP.Work(SSSMI.Backgroundog))
            {
                string InfoPath = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, SMR.SucroseInfo);

                if (File.Exists(InfoPath) && SSTHI.CheckJson(SSTHI.ReadInfo(InfoPath)))
                {
                    SSTHI Info = SSTHI.ReadJson(InfoPath);

                    if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                    {
                        SWUD.RefreshDesktop();

                        if (SMMM.PerformanceCounter)
                        {
                            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Backgroundog}{SMR.ValueSeparator}{SSSMI.Backgroundog}");
                        }

                        switch (Info.Type)
                        {
                            case SSDEWT.Web:
                                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[SSDMM.WApp]}");
                                break;
                            case SSDEWT.Url:
                                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[SSDMM.UApp]}");
                                break;
                            case SSDEWT.Gif:
                                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[SSDMM.GApp]}");
                                break;
                            case SSDEWT.Video:
                                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[SSDMM.VApp]}");
                                break;
                            case SSDEWT.YouTube:
                                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[SSDMM.YApp]}");
                                break;
                            case SSDEWT.Application:
                                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[SSDMM.AApp]}");
                                break;
                            default:
                                break;
                        }
                    }
                }
            }
        }
    }
}