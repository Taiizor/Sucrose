using System.IO;
using SHV = Skylark.Helper.Versionly;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSLMM = Sucrose.Shared.Live.Manage.Manager;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Shared.Live.Helper
{
    internal static class Run
    {
        public static void Start()
        {
            string InfoPath = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, SMR.SucroseInfo);

            if (File.Exists(InfoPath))
            {
                SSTHI Info = SSTHI.ReadJson(InfoPath);

                if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                {
                    switch (Info.Type)
                    {
                        case SSDEWT.Web:
                            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[SSLMM.WApp]}");
                            break;
                        case SSDEWT.Url:
                            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[SSLMM.UApp]}");
                            break;
                        case SSDEWT.Gif:
                            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[SSLMM.GApp]}");
                            break;
                        case SSDEWT.Video:
                            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[SSLMM.VApp]}");
                            break;
                        case SSDEWT.YouTube:
                            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[SSLMM.YApp]}");
                            break;
                        case SSDEWT.Application:
                            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[SSLMM.AApp]}");
                            break;
                        default:
                            break;
                    }
                }
            }
        }
    }
}