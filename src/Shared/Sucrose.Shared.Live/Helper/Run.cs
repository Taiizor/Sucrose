using System.IO;
using SHV = Skylark.Helper.Versionly;
using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SMML = Sucrose.Manager.Manage.Library;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSDMME = Sucrose.Shared.Dependency.Manage.Manager.Engine;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SWUD = Skylark.Wing.Utility.Desktop;

namespace Sucrose.Shared.Live.Helper
{
    internal static class Run
    {
        public static void Start()
        {
            if ((!SMMB.ClosePerformance && !SMMB.PausePerformance) || !SSSHP.Work(SSSMI.Backgroundog))
            {
                string InfoPath = Path.Combine(SMML.LibraryLocation, SMML.LibrarySelected, SMR.SucroseInfo);

                if (File.Exists(InfoPath) && SSTHI.ReadCheck(InfoPath))
                {
                    SSTHI Info = SSTHI.ReadJson(InfoPath);

                    if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                    {
                        SWUD.RefreshDesktop();

                        if (SMMB.PerformanceCounter)
                        {
                            SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Backgroundog}{SMMRG.ValueSeparator}{SSSMI.Backgroundog}");
                        }

                        switch (Info.Type)
                        {
                            case SSDEWT.Gif:
                                SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Live}{SMMRG.ValueSeparator}{SSSMI.EngineLive[SSDMME.GifEngine]}");
                                break;
                            case SSDEWT.Url:
                                SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Live}{SMMRG.ValueSeparator}{SSSMI.EngineLive[SSDMME.UrlEngine]}");
                                break;
                            case SSDEWT.Web:
                                SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Live}{SMMRG.ValueSeparator}{SSSMI.EngineLive[SSDMME.WebEngine]}");
                                break;
                            case SSDEWT.Video:
                                SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Live}{SMMRG.ValueSeparator}{SSSMI.EngineLive[SSDMME.VideoEngine]}");
                                break;
                            case SSDEWT.YouTube:
                                SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Live}{SMMRG.ValueSeparator}{SSSMI.EngineLive[SSDMME.YouTubeEngine]}");
                                break;
                            case SSDEWT.Application:
                                SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Live}{SMMRG.ValueSeparator}{SSSMI.EngineLive[SSDMME.ApplicationEngine]}");
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