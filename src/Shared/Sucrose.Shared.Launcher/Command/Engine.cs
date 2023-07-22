using System.IO;
using SDECT = Sucrose.Dependency.Enum.CommandsType;
using SDEET = Sucrose.Dependency.Enum.EngineType;
using SDEWT = Sucrose.Dependency.Enum.WallpaperType;
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

        private static SDEET AApp => SMMI.EngineSettingManager.GetSetting(SMC.AApp, (SDEET)SSMI.ApplicationEngine);

        private static SDEET YApp => SMMI.EngineSettingManager.GetSetting(SMC.YApp, (SDEET)SSMI.YouTubeEngine);

        private static SDEET VApp => SMMI.EngineSettingManager.GetSetting(SMC.VApp, (SDEET)SSMI.VideoEngine);

        private static SDEET GApp => SMMI.EngineSettingManager.GetSetting(SMC.GApp, (SDEET)SSMI.GifEngine);

        private static SDEET UApp => SMMI.EngineSettingManager.GetSetting(SMC.UApp, (SDEET)SSMI.UrlEngine);

        private static SDEET WApp => SMMI.EngineSettingManager.GetSetting(SMC.WApp, (SDEET)SSMI.WebEngine);

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
                        case SDEWT.Web:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SDECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[WApp]}");
                            break;
                        case SDEWT.Url:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SDECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[UApp]}");
                            break;
                        case SDEWT.Gif:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SDECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[GApp]}");
                            break;
                        case SDEWT.Video:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SDECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[VApp]}");
                            break;
                        case SDEWT.YouTube:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SDECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[YApp]}");
                            break;
                        case SDEWT.Application:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SDECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[AApp]}");
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