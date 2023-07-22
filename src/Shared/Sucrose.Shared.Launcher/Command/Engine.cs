using System.IO;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSHL = Sucrose.Space.Helper.Live;
using SSHP = Sucrose.Space.Helper.Processor;
using SSLCI = Sucrose.Shared.Launcher.Command.Interface;
using SSMI = Sucrose.Space.Manage.Internal;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SWUD = Skylark.Wing.Utility.Desktop;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Engine
    {
        private static string Directory => SMMI.EngineSettingManager.GetSetting(SMC.Directory, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        private static SSDEET AApp => SMMI.EngineSettingManager.GetSetting(SMC.AApp, (SSDEET)SSMI.ApplicationEngine);

        private static SSDEET YApp => SMMI.EngineSettingManager.GetSetting(SMC.YApp, (SSDEET)SSMI.YouTubeEngine);

        private static SSDEET VApp => SMMI.EngineSettingManager.GetSetting(SMC.VApp, (SSDEET)SSMI.VideoEngine);

        private static SSDEET GApp => SMMI.EngineSettingManager.GetSetting(SMC.GApp, (SSDEET)SSMI.GifEngine);

        private static SSDEET UApp => SMMI.EngineSettingManager.GetSetting(SMC.UApp, (SSDEET)SSMI.UrlEngine);

        private static SSDEET WApp => SMMI.EngineSettingManager.GetSetting(SMC.WApp, (SSDEET)SSMI.WebEngine);

        private static string Folder => SMMI.EngineSettingManager.GetSetting(SMC.Folder, string.Empty);

        private static string App => SMMI.AuroraSettingManager.GetSetting(SMC.App, string.Empty);

        public static void Command(bool State = true)
        {
            if (State && SSHL.Run())
            {
                SSHL.Kill();

                if (!string.IsNullOrEmpty(App))
                {
                    SSHP.Kill(App);
                }

                SWUD.RefreshDesktop();

                SMMI.AuroraSettingManager.SetSetting(SMC.App, string.Empty);
            }
            else if (!SSHL.Run() && SMMI.EngineSettingManager.CheckFile() && !string.IsNullOrEmpty(Folder))
            {
                string InfoPath = Path.Combine(Directory, Folder, SMR.SucroseInfo);

                if (File.Exists(InfoPath))
                {
                    SSTHI Info = SSTHI.ReadJson(InfoPath);

                    switch (Info.Type)
                    {
                        case SSDEWT.Web:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[WApp]}");
                            break;
                        case SSDEWT.Url:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[UApp]}");
                            break;
                        case SSDEWT.Gif:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[GApp]}");
                            break;
                        case SSDEWT.Video:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[VApp]}");
                            break;
                        case SSDEWT.YouTube:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[YApp]}");
                            break;
                        case SSDEWT.Application:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[AApp]}");
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