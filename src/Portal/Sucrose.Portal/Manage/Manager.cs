using System.IO;
using Wpf.Ui.Controls;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHC = Skylark.Helper.Culture;
using SHS = Skylark.Helper.Skymath;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Portal.Manage
{
    internal static class Manager
    {
        public static IList<char> Chars => Enumerable.Range('A', 'Z' - 'A' + 1).Concat(Enumerable.Range('a', 'z' - 'a' + 1)).Concat(Enumerable.Range('0', '9' - '0' + 1)).Select(C => (char)C).ToList();

        public static string LibraryLocation => SMMI.LibrarySettingManager.GetSetting(SMC.LibraryLocation, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        public static int DescriptionLength => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.DescriptionLength, 30), 10, 100);

        public static int LibraryPagination => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.LibraryPagination, 36), 1, 100);

        public static string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SHC.CurrentUITwoLetterISOLanguageName);

        public static WindowBackdropType BackdropType => SMMI.PortalSettingManager.GetSetting(SMC.BackdropType, DefaultBackdropType);

        public static int StorePagination => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.StorePagination, 30), 1, 100);

        public static int AdaptiveLayout => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.AdaptiveLayout, 0), 0, 100);

        public static int AdaptiveMargin => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.AdaptiveMargin, 8), 5, 25);

        public static int TitleLength => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.TitleLength, 25), 10, 100);

        public static string LibrarySelected => SMMI.LibrarySettingManager.GetSetting(SMC.LibrarySelected, string.Empty);

        public static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        public static string Agent => SMMI.GeneralSettingManager.GetSetting(SMC.UserAgent, SMR.UserAgent);

        public static string Key => SMMI.PrivateSettingManager.GetSetting(SMC.Key, SMR.Key);

        public static bool Start => SMMI.EngineSettingManager.GetSetting(SMC.Start, false);

        public static bool Adult => SMMI.PortalSettingManager.GetSetting(SMC.Adult, false);

        public static WindowBackdropType DefaultBackdropType => WindowBackdropType.None;

        public static Mutex Mutex => new(true, SMR.PortalMutex);
    }
}