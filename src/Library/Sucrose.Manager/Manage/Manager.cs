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

namespace Sucrose.Manager.Manage
{
    public static class Manager
    {
        public static IList<char> Chars => Enumerable.Range('A', 'Z' - 'A' + 1).Concat(Enumerable.Range('a', 'z' - 'a' + 1)).Concat(Enumerable.Range('0', '9' - '0' + 1)).Select(C => (char)C).ToList();

        public static string LibraryLocation => SMMI.LibrarySettingManager.GetSetting(SMC.LibraryLocation, Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.Library));

        public static Dictionary<string, string> CefArguments => SMMI.EngineSettingManager.GetSetting(SMC.CefArguments, new Dictionary<string, string>());

        public static int UpdateLimitValue => SHS.Clamp(SMMI.UpdateSettingManager.GetSettingStable(SMC.UpdateLimitValue, 500), 0, 99999999);

        public static int DownloadValue => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMC.DownloadValue, 10), 0, 99999999);

        public static int BackgroundOpacity => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.BackgroundOpacity, 100), 0, 100);

        public static int DescriptionLength => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.DescriptionLength, 25), 10, 100);

        public static int LibraryPagination => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.LibraryPagination, 30), 1, 100);

        public static int UploadValue => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMC.UploadValue, 800), 0, 99999999);

        public static string[] GraphicInterfaces => SMMI.SystemSettingManager.GetSetting(SMC.GraphicInterfaces, Array.Empty<string>());

        public static string[] NetworkInterfaces => SMMI.SystemSettingManager.GetSetting(SMC.NetworkInterfaces, Array.Empty<string>());

        public static int AdvertisingDelay => SHS.Clamp(SMMI.DonateSettingManager.GetSettingStable(SMC.AdvertisingDelay, 30), 30, 720);

        public static int PassingCycyling => SHS.Clamp(SMMI.CyclingSettingManager.GetSettingStable(SMC.PassingCycyling, 0), 0, 99999);

        public static List<string> DisableCycyling => SMMI.CyclingSettingManager.GetSetting(SMC.DisableCycyling, new List<string>());

        public static int StorePagination => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.StorePagination, 30), 1, 100);

        public static int BatteryUsage => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMC.BatteryUsage, 50), 0, 100);

        public static int MemoryUsage => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMC.MemoryUsage, 80), 0, 100);

        public static SEDEST DuplicateScreenType => SMMI.EngineSettingManager.GetSetting(SMC.DuplicateScreenType, SEDEST.Default);

        public static string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.Culture, SHC.CurrentUITwoLetterISOLanguageName);

        public static SEDYST DisplayScreenType => SMMI.EngineSettingManager.GetSetting(SMC.DisplayScreenType, SEDYST.PerDisplay);

        public static int DeveloperPort => SHS.Clamp(SMMI.EngineSettingManager.GetSettingStable(SMC.DeveloperPort, 0), 0, 65535);

        public static int AdaptiveLayout => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.AdaptiveLayout, 0), 0, 100);

        public static int PingValue => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMC.PingValue, 100), 0, 1000);

        public static int AdaptiveMargin => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.AdaptiveMargin, 5), 5, 25);

        public static int CycylingTime => SHS.Clamp(SMMI.CyclingSettingManager.GetSettingStable(SMC.CycylingTime, 30), 1, 999);

        public static List<string> WebArguments => SMMI.EngineSettingManager.GetSetting(SMC.WebArguments, new List<string>());

        public static int StoreDuration => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.StoreDuration, 3), 1, 24);

        public static int DiscordDelay => SHS.Clamp(SMMI.HookSettingManager.GetSettingStable(SMC.DiscordDelay, 60), 60, 3600);

        public static int TitleLength => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.TitleLength, 20), 10, 100);

        public static int GpuUsage => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMC.GpuUsage, 70), 0, 100);

        public static int CpuUsage => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMC.CpuUsage, 70), 0, 100);

        public static string GraphicAdapter => SMMI.BackgroundogSettingManager.GetSetting(SMC.GraphicAdapter, string.Empty);

        public static string NetworkAdapter => SMMI.BackgroundogSettingManager.GetSetting(SMC.NetworkAdapter, string.Empty);

        public static int ScreenIndex => SHS.Clamp(SMMI.EngineSettingManager.GetSettingStable(SMC.ScreenIndex, 0), 0, 100);

        public static bool PerformanceCounter => SMMI.BackgroundogSettingManager.GetSetting(SMC.PerformanceCounter, true);

        public static SEEST ExpandScreenType => SMMI.EngineSettingManager.GetSetting(SMC.ExpandScreenType, SEEST.Default);

        public static SESET UpdateLimitType => SMMI.UpdateSettingManager.GetSetting(SMC.UpdateLimitType, SESET.Megabyte);

        public static string LibrarySelected => SMMI.LibrarySettingManager.GetSetting(SMC.LibrarySelected, string.Empty);

        public static SESET DownloadType => SMMI.BackgroundogSettingManager.GetSetting(SMC.DownloadType, SESET.Megabyte);

        public static string BackgroundImage => SMMI.PortalSettingManager.GetSetting(SMC.BackgroundImage, string.Empty);

        public static bool PausePerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.PausePerformance, false);

        public static bool ClosePerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.ClosePerformance, false);

        public static SESET UploadType => SMMI.BackgroundogSettingManager.GetSetting(SMC.UploadType, SESET.Kilobyte);

        public static bool LibraryPreviewHide => SMMI.PortalSettingManager.GetSetting(SMC.LibraryPreviewHide, false);

        public static List<string> Showcase => SMMI.UserSettingManager.GetSetting(SMC.Showcase, new List<string>());

        public static DateTime CefSharpTime => SMMI.UserSettingManager.GetSetting(SMC.CefSharpTime, new DateTime());

        public static int Volume => SHS.Clamp(SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100), 0, 100);

        public static int Startup => SHS.Clamp(SMMI.GeneralSettingManager.GetSettingStable(SMC.Startup, 0), 0, 10);

        public static bool SignalRequired => SMMI.BackgroundogSettingManager.GetSetting(SMC.SignalRequired, false);

        public static SESNT ScreenType => SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SESNT.DisplayBound);

        public static DateTime WebViewTime => SMMI.UserSettingManager.GetSetting(SMC.WebViewTime, new DateTime());

        public static DateTime UpdateTime => SMMI.UpdateSettingManager.GetSetting(SMC.UpdateTime, new DateTime());

        public static List<string> Themes => SMMI.ThemesSettingManager.GetSetting(SMC.Themes, new List<string>());

        public static bool StorePreviewHide => SMMI.PortalSettingManager.GetSetting(SMC.StorePreviewHide, false);

        public static bool AudioRequired => SMMI.BackgroundogSettingManager.GetSetting(SMC.AudioRequired, false);

        public static bool AdvertisingState => SMMI.DonateSettingManager.GetSetting(SMC.AdvertisingState, true);

        public static bool PipeRequired => SMMI.BackgroundogSettingManager.GetSetting(SMC.PipeRequired, false);

        public static string UserAgent => SMMI.GeneralSettingManager.GetSetting(SMC.UserAgent, SMR.UserAgent);

        public static bool LibraryConfirm => SMMI.LibrarySettingManager.GetSetting(SMC.LibraryConfirm, true);

        public static bool LibraryPreview => SMMI.PortalSettingManager.GetSetting(SMC.LibraryPreview, false);

        public static bool LibraryDelete => SMMI.LibrarySettingManager.GetSetting(SMC.LibraryDelete, false);

        public static SEIT InputType => SMMI.EngineSettingManager.GetSetting(SMC.InputType, SEIT.OnlyMouse);

        public static bool DeveloperMode => SMMI.EngineSettingManager.GetSetting(SMC.DeveloperMode, false);

        public static bool VolumeDesktop => SMMI.EngineSettingManager.GetSetting(SMC.VolumeDesktop, false);

        public static string PingType => SMMI.BackgroundogSettingManager.GetSetting(SMC.PingType, "Bing");

        public static bool DonateVisible => SMMI.DonateSettingManager.GetSetting(SMC.DonateVisible, true);

        public static bool DiscordRefresh => SMMI.HookSettingManager.GetSetting(SMC.DiscordRefresh, true);

        public static bool StorePreview => SMMI.PortalSettingManager.GetSetting(SMC.StorePreview, false);

        public static bool InputDesktop => SMMI.EngineSettingManager.GetSetting(SMC.InputDesktop, false);

        public static bool LibraryStart => SMMI.EngineSettingManager.GetSetting(SMC.LibraryStart, true);

        public static bool UpdateState => SMMI.UpdateSettingManager.GetSetting(SMC.UpdateState, false);

        public static bool LibraryMove => SMMI.LibrarySettingManager.GetSetting(SMC.LibraryMove, true);

        public static bool DiscordState => SMMI.HookSettingManager.GetSetting(SMC.DiscordState, true);

        public static bool Statistics => SMMI.GeneralSettingManager.GetSetting(SMC.Statistics, true);

        public static bool StoreStart => SMMI.EngineSettingManager.GetSetting(SMC.StoreStart, true);

        public static bool Cycyling => SMMI.CyclingSettingManager.GetSetting(SMC.Cycyling, false);

        public static bool Visible => SMMI.LauncherSettingManager.GetSetting(SMC.Visible, true);

        public static string App => SMMI.AuroraSettingManager.GetSetting(SMC.App, string.Empty);

        public static bool Shuffle => SMMI.EngineSettingManager.GetSetting(SMC.Shuffle, true);

        public static bool Report => SMMI.GeneralSettingManager.GetSetting(SMC.Report, true);

        public static string Key => SMMI.PrivateSettingManager.GetSetting(SMC.Key, SMR.Key);

        public static bool Exit => SMMI.LauncherSettingManager.GetSetting(SMC.Exit, false);

        public static bool Adult => SMMI.PortalSettingManager.GetSetting(SMC.Adult, true);

        public static bool Loop => SMMI.EngineSettingManager.GetSetting(SMC.Loop, true);
    }
}