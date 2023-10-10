using System.IO;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSCECT = Sucrose.Shared.Core.Enum.ChannelType;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Update.Manage
{
    internal static class Manager
    {
        public static string CachePath => Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Bundle);

        public static SSCECT ChannelType => SMMI.UpdateSettingManager.GetSetting(SMC.ChannelType, SSCECT.Release);

        public static SEWTT ThemeType => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());
    }
}