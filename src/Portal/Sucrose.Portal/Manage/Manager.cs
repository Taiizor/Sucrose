using System.IO;
using System.Net;
using System.Windows.Media;
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

        public static Stretch BackgroundStretch => SMMI.PortalSettingManager.GetSetting(SMC.BackgroundStretch, DefaultBackgroundStretch);

        public static int LibraryPagination => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.LibraryPagination, 36), 1, 100);

        public static string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SHC.CurrentUITwoLetterISOLanguageName);

        public static WindowBackdropType BackdropType => SMMI.PortalSettingManager.GetSetting(SMC.BackdropType, DefaultBackdropType);

        public static int StorePagination => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.StorePagination, 30), 1, 100);

        public static int AdaptiveLayout => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.AdaptiveLayout, 0), 0, 100);

        public static int AdaptiveMargin => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.AdaptiveMargin, 8), 5, 25);

        public static int DiscordDelay => SHS.Clamp(SMMI.HookSettingManager.GetSettingStable(SMC.DiscordDelay, 60), 60, 3600);

        public static int TitleLength => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.TitleLength, 25), 10, 100);

        public static string LibrarySelected => SMMI.LibrarySettingManager.GetSetting(SMC.LibrarySelected, string.Empty);

        public static string BackgroundImage => SMMI.PortalSettingManager.GetSetting(SMC.BackgroundImage, string.Empty);

        public static int BackgroundOpacity => SMMI.PortalSettingManager.GetSettingStable(SMC.BackgroundOpacity, 100);

        public static IPAddress Host => SMMI.LauncherSettingManager.GetSettingAddress(SMC.Host, IPAddress.Loopback);

        public static bool LibraryDelete => SMMI.LibrarySettingManager.GetSetting(SMC.LibraryDelete, false);


        public static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        public static bool VolumeDesktop => SMMI.EngineSettingManager.GetSetting(SMC.VolumeDesktop, false);

        public static string Agent => SMMI.GeneralSettingManager.GetSetting(SMC.UserAgent, SMR.UserAgent);

        public static bool DiscordRefresh => SMMI.HookSettingManager.GetSetting(SMC.DiscordRefresh, true);

        public static bool LibraryMove => SMMI.LibrarySettingManager.GetSetting(SMC.LibraryMove, true);

        public static bool DiscordState => SMMI.HookSettingManager.GetSetting(SMC.DiscordState, true);

        public static int Startup => SMMI.GeneralSettingManager.GetSettingStable(SMC.Startup, 0);

        public static bool Visible => SMMI.LauncherSettingManager.GetSetting(SMC.Visible, true);

        public static string App => SMMI.AuroraSettingManager.GetSetting(SMC.App, string.Empty);

        public static int Volume => SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 0);

        public static int Port => SMMI.LauncherSettingManager.GetSettingStable(SMC.Port, 0);

        public static string Key => SMMI.PrivateSettingManager.GetSetting(SMC.Key, SMR.Key);

        public static bool Start => SMMI.EngineSettingManager.GetSetting(SMC.Start, false);

        public static bool Adult => SMMI.PortalSettingManager.GetSetting(SMC.Adult, false);

        public static WindowBackdropType DefaultBackdropType => WindowBackdropType.None;

        public static Stretch DefaultBackgroundStretch => Stretch.UniformToFill;

        public static Mutex Mutex => new(true, SMR.PortalMutex);
    }
}