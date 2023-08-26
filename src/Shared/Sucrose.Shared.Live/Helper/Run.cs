using System.IO;
using SHV = Skylark.Helper.Versionly;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Shared.Live.Helper
{
    internal static class Run
    {
        private static string LibraryLocation => SMMI.LibrarySettingManager.GetSetting(SMC.LibraryLocation, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        private static string LibrarySelected => SMMI.LibrarySettingManager.GetSetting(SMC.LibrarySelected, string.Empty);

        private static SSDEET AApp => SMMI.EngineSettingManager.GetSetting(SMC.AApp, (SSDEET)SSSMI.ApplicationEngine);

        private static SSDEET YApp => SMMI.EngineSettingManager.GetSetting(SMC.YApp, (SSDEET)SSSMI.YouTubeEngine);

        private static SSDEET VApp => SMMI.EngineSettingManager.GetSetting(SMC.VApp, (SSDEET)SSSMI.VideoEngine);

        private static SSDEET GApp => SMMI.EngineSettingManager.GetSetting(SMC.GApp, (SSDEET)SSSMI.GifEngine);

        private static SSDEET UApp => SMMI.EngineSettingManager.GetSetting(SMC.UApp, (SSDEET)SSSMI.UrlEngine);

        private static SSDEET WApp => SMMI.EngineSettingManager.GetSetting(SMC.WApp, (SSDEET)SSSMI.WebEngine);

        public static void Start()
        {
            string InfoPath = Path.Combine(LibraryLocation, LibrarySelected, SMR.SucroseInfo);

            if (File.Exists(InfoPath))
            {
                SSTHI Info = SSTHI.ReadJson(InfoPath);

                if (Info.AppVersion.CompareTo(SHV.Entry()) <= 0)
                {
                    switch (Info.Type)
                    {
                        case SSDEWT.Web:
                            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[WApp]}");
                            break;
                        case SSDEWT.Url:
                            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[UApp]}");
                            break;
                        case SSDEWT.Gif:
                            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[GApp]}");
                            break;
                        case SSDEWT.Video:
                            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[VApp]}");
                            break;
                        case SSDEWT.YouTube:
                            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[YApp]}");
                            break;
                        case SSDEWT.Application:
                            SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSSMI.EngineLive[AApp]}");
                            break;
                        default:
                            break;
                    }
                }
            }
        }
    }
}