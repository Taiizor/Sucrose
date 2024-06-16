using System.IO;
using Wpf.Ui.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Portal.Manage
{
    internal static class Manager
    {
        public static WindowBackdropType BackdropType => SMMI.PortalSettingManager.GetSetting(SMC.BackdropType, DefaultBackdropType);

        public static string AlternativeLibrary => Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.Library);

        public static WindowBackdropType DefaultBackdropType => WindowBackdropType.None;
    }
}