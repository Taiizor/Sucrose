using SEDEST = Skylark.Enum.DuplicateScreenType;
using SEDYST = Skylark.Enum.DisplayScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEIT = Skylark.Enum.InputType;
using SESET = Skylark.Enum.StorageType;
using SESNT = Skylark.Enum.ScreenType;
using SHC = Skylark.Helper.Culture;
using SHS = Skylark.Helper.Skymath;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMMCH = Sucrose.Memory.Manage.Constant.Hook;
using SMMCU = Sucrose.Memory.Manage.Constant.User;
using SMMCA = Sucrose.Memory.Manage.Constant.Aurora;
using SMMCD = Sucrose.Memory.Manage.Constant.Donate;

namespace Sucrose.Manager.Manage
{
    public static class Manager
    {
        public static IList<char> Chars => Enumerable.Range('A', 'Z' - 'A' + 1).Concat(Enumerable.Range('a', 'z' - 'a' + 1)).Concat(Enumerable.Range('0', '9' - '0' + 1)).Select(C => (char)C).ToList();

        public static int BackgroundOpacity => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.BackgroundOpacity, 100), 0, 100);

        public static int AdvertisingDelay => SHS.Clamp(SMMI.DonateSettingManager.GetSettingStable(SMMCD.AdvertisingDelay, 30), 30, 720);

        public static int LibraryPagination => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.LibraryPagination, 30), 1, 100);

        public static string[] GraphicInterfaces => SMMI.SystemSettingManager.GetSetting(SMC.GraphicInterfaces, Array.Empty<string>());

        public static string[] NetworkInterfaces => SMMI.SystemSettingManager.GetSetting(SMC.NetworkInterfaces, Array.Empty<string>());

        public static int PassingCycyling => SHS.Clamp(SMMI.CyclingSettingManager.GetSettingStable(SMC.PassingCycyling, 0), 0, 99999);

        public static List<string> DisableCycyling => SMMI.CyclingSettingManager.GetSetting(SMC.DisableCycyling, new List<string>());

        public static int StorePagination => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.StorePagination, 30), 1, 100);

        public static string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.Culture, SHC.CurrentUITwoLetterISOLanguageName);

        public static int AdaptiveLayout => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.AdaptiveLayout, 0), 0, 100);

        public static int DiscordDelay => SHS.Clamp(SMMI.HookSettingManager.GetSettingStable(SMMCH.DiscordDelay, 60), 60, 3600);

        public static int AdaptiveMargin => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.AdaptiveMargin, 5), 5, 25);

        public static int CycylingTime => SHS.Clamp(SMMI.CyclingSettingManager.GetSettingStable(SMC.CycylingTime, 30), 1, 999);

        public static int StoreDuration => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.StoreDuration, 3), 1, 24);

        public static string BackgroundImage => SMMI.PortalSettingManager.GetSetting(SMC.BackgroundImage, string.Empty);

        public static string AppProcessName => SMMI.AuroraSettingManager.GetSetting(SMMCA.AppProcessName, string.Empty);

        public static List<string> Showcase => SMMI.UserSettingManager.GetSetting(SMMCU.Showcase, new List<string>());

        public static DateTime CefSharpTime => SMMI.UserSettingManager.GetSetting(SMMCU.CefSharpTime, new DateTime());

        public static bool LibraryPreviewHide => SMMI.PortalSettingManager.GetSetting(SMC.LibraryPreviewHide, false);

        public static DateTime WebViewTime => SMMI.UserSettingManager.GetSetting(SMMCU.WebViewTime, new DateTime());

        public static int Startup => SHS.Clamp(SMMI.GeneralSettingManager.GetSettingStable(SMC.Startup, 0), 0, 10);

        public static bool AdvertisingState => SMMI.DonateSettingManager.GetSetting(SMMCD.AdvertisingState, true);

        public static bool StorePreviewHide => SMMI.PortalSettingManager.GetSetting(SMC.StorePreviewHide, false);

        public static bool CefsharpContinue => SMMI.UserSettingManager.GetSetting(SMMCU.CefsharpContinue, false);

        public static bool WebViewContinue => SMMI.UserSettingManager.GetSetting(SMMCU.WebViewContinue, false);

        public static string UserAgent => SMMI.GeneralSettingManager.GetSetting(SMC.UserAgent, SMR.UserAgent);

        public static bool LibraryPreview => SMMI.PortalSettingManager.GetSetting(SMC.LibraryPreview, false);

        public static bool DonateVisible => SMMI.DonateSettingManager.GetSetting(SMMCD.DonateVisible, true);

        public static bool DiscordRefresh => SMMI.HookSettingManager.GetSetting(SMMCH.DiscordRefresh, true);

        public static bool StorePreview => SMMI.PortalSettingManager.GetSetting(SMC.StorePreview, false);

        public static bool HintTrayIcon => SMMI.UserSettingManager.GetSetting(SMMCU.HintTrayIcon, true);

        public static bool DiscordState => SMMI.HookSettingManager.GetSetting(SMMCH.DiscordState, true);

        public static bool Statistics => SMMI.GeneralSettingManager.GetSetting(SMC.Statistics, true);

        public static bool Cycyling => SMMI.CyclingSettingManager.GetSetting(SMC.Cycyling, false);

        public static bool AppExit => SMMI.LauncherSettingManager.GetSetting(SMC.AppExit, false);

        public static bool Visible => SMMI.LauncherSettingManager.GetSetting(SMC.Visible, true);

        public static bool Report => SMMI.GeneralSettingManager.GetSetting(SMC.Report, true);

        public static string Key => SMMI.PrivateSettingManager.GetSetting(SMC.Key, SMR.Key);

        public static bool Adult => SMMI.PortalSettingManager.GetSetting(SMC.Adult, true);
    }
}