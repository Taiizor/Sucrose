using System.IO;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSLCI = Sucrose.Shared.Launcher.Command.Interface;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SWUD = Skylark.Wing.Utility.Desktop;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Engine
    {
        private static string Directory => SMMI.EngineSettingManager.GetSetting(SMC.Directory, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        private static SSDEET AApp => SMMI.EngineSettingManager.GetSetting(SMC.AApp, (SSDEET)SSSMI.ApplicationEngine);

        private static SSDEET YApp => SMMI.EngineSettingManager.GetSetting(SMC.YApp, (SSDEET)SSSMI.YouTubeEngine);

        private static SSDEET VApp => SMMI.EngineSettingManager.GetSetting(SMC.VApp, (SSDEET)SSSMI.VideoEngine);

        private static SSDEET GApp => SMMI.EngineSettingManager.GetSetting(SMC.GApp, (SSDEET)SSSMI.GifEngine);

        private static SSDEET UApp => SMMI.EngineSettingManager.GetSetting(SMC.UApp, (SSDEET)SSSMI.UrlEngine);

        private static SSDEET WApp => SMMI.EngineSettingManager.GetSetting(SMC.WApp, (SSDEET)SSSMI.WebEngine);

        private static string Folder => SMMI.EngineSettingManager.GetSetting(SMC.Folder, string.Empty);

        private static string App => SMMI.AuroraSettingManager.GetSetting(SMC.App, string.Empty);

        public static void Command(bool State = true)
        {
            if (State && SSSHL.Run())
            {
                SSSHL.Kill();

                if (!string.IsNullOrEmpty(App))
                {
                    SSSHP.Kill(App);
                }

                SWUD.RefreshDesktop();

                SMMI.AuroraSettingManager.SetSetting(SMC.App, string.Empty);
            }
            else if (!SSSHL.Run() && SMMI.EngineSettingManager.CheckFile() && !string.IsNullOrEmpty(Folder))
            {
                string InfoPath = Path.Combine(Directory, Folder, SMR.SucroseInfo);

                if (File.Exists(InfoPath))
                {
                    SSTHI Info = SSTHI.ReadJson(InfoPath);

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
            else if (!SMMI.EngineSettingManager.CheckFile() || string.IsNullOrEmpty(Folder))
            {
                SSLCI.Command();
            }
        }
    }
}