using System.IO;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SSEET = Sucrose.Space.Enum.EngineType;
using SSHC = Sucrose.Space.Helper.Command;
using SSHL = Sucrose.Space.Helper.Live;
using SSMI = Sucrose.Space.Manage.Internal;
using SWUD = Skylark.Wing.Utility.Desktop;
using STSHI = Sucrose.Theme.Shared.Helper.Info;
using SEWT = Skylark.Enum.WallpaperType;

namespace Sucrose.Tray.Command
{
    internal static class Engine
    {
        private static string Directory => SMMI.EngineSettingManager.GetSetting(SMC.Directory, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        private static SSEET AApp => SMMI.EngineSettingManager.GetSetting(SMC.AApp, SSMI.WallpaperLive[SEWT.Application]);

        private static SSEET YApp => SMMI.EngineSettingManager.GetSetting(SMC.YApp, SSMI.WallpaperLive[SEWT.YouTube]);

        private static SSEET VApp => SMMI.EngineSettingManager.GetSetting(SMC.VApp, SSMI.WallpaperLive[SEWT.Video]);

        private static SSEET GApp => SMMI.EngineSettingManager.GetSetting(SMC.GApp, SSMI.WallpaperLive[SEWT.Gif]);

        private static SSEET UApp => SMMI.EngineSettingManager.GetSetting(SMC.UApp, SSMI.WallpaperLive[SEWT.Url]);

        private static SSEET WApp => SMMI.EngineSettingManager.GetSetting(SMC.WApp, SSMI.WallpaperLive[SEWT.Web]);

        private static string Folder => SMMI.EngineSettingManager.GetSetting(SMC.Folder, string.Empty);

        public static void Command(bool State = true)
        {
            if (State && SSHL.Run())
            {
                SSHL.Kill();

                SWUD.RefreshDesktop();
            }
            else if (SMMI.EngineSettingManager.CheckFile() && !string.IsNullOrEmpty(Folder))
            {
                string InfoPath = Path.Combine(Directory, Folder, SMR.SucroseInfo);

                if (File.Exists(InfoPath))
                {
                    STSHI Info = STSHI.ReadJson(InfoPath);

                    switch (Info.Type)
                    {
                        case SEWT.Web:
                            SSHC.Run(SSMI.CommandLine, $"{SMR.StartCommand}{SSECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[WApp]}");
                            break;
                        case SEWT.Url:
                            SSHC.Run(SSMI.CommandLine, $"{SMR.StartCommand}{SSECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[UApp]}");
                            break;
                        case SEWT.Gif:
                            SSHC.Run(SSMI.CommandLine, $"{SMR.StartCommand}{SSECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[GApp]}");
                            break;
                        case SEWT.Video:
                            SSHC.Run(SSMI.CommandLine, $"{SMR.StartCommand}{SSECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[VApp]}");
                            break;
                        case SEWT.YouTube:
                            SSHC.Run(SSMI.CommandLine, $"{SMR.StartCommand}{SSECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[YApp]}");
                            break;
                        case SEWT.Application:
                            SSHC.Run(SSMI.CommandLine, $"{SMR.StartCommand}{SSECT.Live}{SMR.ValueSeparator}{SSMI.EngineLive[AApp]}");
                            break;
                        default:
                            break;
                    }
                }
            }
        }
    }
}