using System.IO;
using SSEWT = Sucrose.Space.Enum.WallpaperType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SSEET = Sucrose.Space.Enum.EngineType;
using SSHP = Sucrose.Space.Helper.Processor;
using SSHL = Sucrose.Space.Helper.Live;
using SSMI = Sucrose.Space.Manage.Internal;
using STCI = Sucrose.Tray.Command.Interface;
using STSHI = Sucrose.Theme.Shared.Helper.Info;
using SWUD = Skylark.Wing.Utility.Desktop;

namespace Sucrose.Tray.Command
{
    internal static class Engine
    {
        private static string Directory => SMMI.EngineSettingManager.GetSetting(SMC.Directory, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        private static SSEET AApp => SMMI.EngineSettingManager.GetSetting(SMC.AApp, (SSEET)SSMI.ApplicationEngine);

        private static SSEET YApp => SMMI.EngineSettingManager.GetSetting(SMC.YApp, (SSEET)SSMI.YouTubeEngine);

        private static SSEET VApp => SMMI.EngineSettingManager.GetSetting(SMC.VApp, (SSEET)SSMI.VideoEngine);

        private static SSEET GApp => SMMI.EngineSettingManager.GetSetting(SMC.GApp, (SSEET)SSMI.GifEngine);

        private static SSEET UApp => SMMI.EngineSettingManager.GetSetting(SMC.UApp, (SSEET)SSMI.UrlEngine);

        private static SSEET WApp => SMMI.EngineSettingManager.GetSetting(SMC.WApp, (SSEET)SSMI.WebEngine);

        private static string Folder => SMMI.EngineSettingManager.GetSetting(SMC.Folder, string.Empty);

        public static void Command(bool State = true)
        {
            if (State && SSHL.Run())
            {
                SSHL.Kill();

                SWUD.RefreshDesktop();
            }
            else if (!SSHL.Run() && SMMI.EngineSettingManager.CheckFile() && !string.IsNullOrEmpty(Folder))
            {
                string InfoPath = Path.Combine(Directory, Folder, SMR.SucroseInfo);

                if (File.Exists(InfoPath))
                {
                    STSHI Info = STSHI.ReadJson(InfoPath);

                    switch (Info.Type)
                    {
                        case SSEWT.Web:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SSECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[WApp]}");
                            break;
                        case SSEWT.Url:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SSECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[UApp]}");
                            break;
                        case SSEWT.Gif:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SSECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[GApp]}");
                            break;
                        case SSEWT.Video:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SSECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[VApp]}");
                            break;
                        case SSEWT.YouTube:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SSECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[YApp]}");
                            break;
                        case SSEWT.Application:
                            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SSECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[AApp]}");
                            break;
                        default:
                            break;
                    }
                }
            }
            else if (!SMMI.EngineSettingManager.CheckFile() || string.IsNullOrEmpty(Folder))
            {
                STCI.Command();
            }
        }
    }
}