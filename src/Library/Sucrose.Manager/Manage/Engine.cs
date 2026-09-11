using SEDEST = Skylark.Enum.DuplicateScreenType;
using SEDYST = Skylark.Enum.DisplayScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEIT = Skylark.Enum.InputType;
using SESNT = Skylark.Enum.ScreenType;
using SHS = Skylark.Helper.Skymath;
using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSDEPT = Sucrose.Shared.Dependency.Enum.ProxyType;

namespace Sucrose.Manager.Manage
{
    public static class Engine
    {
        public static Dictionary<string, string> CefArguments => SMMI.EngineSettingManager.GetSetting(SMMCE.CefArguments, new Dictionary<string, string>());

        public static int VolumeSilentSensitivity => SHS.Clamp(SMMI.EngineSettingManager.GetSettingStable(SMMCE.VolumeSilentSensitivity, 5), 1, 10);

        public static int WallpaperVolume => SHS.Clamp(SMMI.EngineSettingManager.GetSettingStable(SMMCE.WallpaperVolume, 100), 0, 100);

        public static SEDEST DuplicateScreenType => SMMI.EngineSettingManager.GetSetting(SMMCE.DuplicateScreenType, SEDEST.Default);

        public static SEDYST DisplayScreenType => SMMI.EngineSettingManager.GetSetting(SMMCE.DisplayScreenType, SEDYST.PerDisplay);

        public static int DeveloperPort => SHS.Clamp(SMMI.EngineSettingManager.GetSettingStable(SMMCE.DeveloperPort, 0), 0, 65535);

        public static string BackgroundImagePath => SMMI.EngineSettingManager.GetSetting(SMMCE.BackgroundImagePath, string.Empty);

        public static List<string> WebArguments => SMMI.EngineSettingManager.GetSetting(SMMCE.WebArguments, new List<string>());

        public static SEEST ExpandScreenType => SMMI.EngineSettingManager.GetSetting(SMMCE.ExpandScreenType, SEEST.Default);

        public static bool HardwareAcceleration => SMMI.EngineSettingManager.GetSetting(SMMCE.HardwareAcceleration, true);

        public static SESNT ScreenType => SMMI.EngineSettingManager.GetSetting(SMMCE.ScreenType, SESNT.DisplayBound);

        public static string ScreenDevice => SMMI.EngineSettingManager.GetSetting(SMMCE.ScreenDevice, string.Empty);

        public static bool WallpaperShuffle => SMMI.EngineSettingManager.GetSetting(SMMCE.WallpaperShuffle, true);

        public static SEIT InputType => SMMI.EngineSettingManager.GetSetting(SMMCE.InputType, SEIT.MouseKeyboard);

        public static bool BackgroundImage => SMMI.EngineSettingManager.GetSetting(SMMCE.BackgroundImage, false);

        public static bool VolumeDesktop => SMMI.EngineSettingManager.GetSetting(SMMCE.VolumeDesktop, false);

        public static bool DeveloperMode => SMMI.EngineSettingManager.GetSetting(SMMCE.DeveloperMode, false);

        public static bool CrashExplorer => SMMI.EngineSettingManager.GetSetting(SMMCE.CrashExplorer, false);

        public static bool WallpaperLoop => SMMI.EngineSettingManager.GetSetting(SMMCE.WallpaperLoop, true);

        public static bool VolumeSilent => SMMI.EngineSettingManager.GetSetting(SMMCE.VolumeSilent, false);

        public static bool InputDesktop => SMMI.EngineSettingManager.GetSetting(SMMCE.InputDesktop, false);

        public static bool LibraryStart => SMMI.EngineSettingManager.GetSetting(SMMCE.LibraryStart, true);

        public static bool StoreStart => SMMI.EngineSettingManager.GetSetting(SMMCE.StoreStart, true);

        public static bool StayAwake => SMMI.EngineSettingManager.GetSetting(SMMCE.StayAwake, false);

        public static bool ProxyEnabled => SMMI.EngineSettingManager.GetSetting(SMMCE.ProxyEnabled, false);

        public static SSDEPT ProxyType => SMMI.EngineSettingManager.GetSetting(SMMCE.ProxyType, SSDEPT.None);

        public static string ProxyServer => SMMI.EngineSettingManager.GetSetting(SMMCE.ProxyServer, string.Empty);

        public static int ProxyPort => SHS.Clamp(SMMI.EngineSettingManager.GetSettingStable(SMMCE.ProxyPort, 0), 0, 65535);

        public static string ProxyUsername => SMMI.EngineSettingManager.GetSetting(SMMCE.ProxyUsername, string.Empty);

        public static string ProxyPassword => SMMI.EngineSettingManager.GetSetting(SMMCE.ProxyPassword, string.Empty);
    }
}