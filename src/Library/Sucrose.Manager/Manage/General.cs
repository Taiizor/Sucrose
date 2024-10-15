using SHC = Skylark.Helper.Culture;
using SHS = Skylark.Helper.Skymath;
using SMMCG = Sucrose.Memory.Manage.Constant.General;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;

namespace Sucrose.Manager.Manage
{
    public static class General
    {
        public static string Culture => SMMI.GeneralSettingManager.GetSetting(SMMCG.Culture, SHC.CurrentUITwoLetterISOLanguageName);

        public static int RunStartup => SHS.Clamp(SMMI.GeneralSettingManager.GetSettingStable(SMMCG.RunStartup, 0), 0, 10);

        public static string UserAgent => SMMI.GeneralSettingManager.GetSetting(SMMCG.UserAgent, SMMRG.UserAgent);

        public static bool TrayIconVisible => SMMI.GeneralSettingManager.GetSetting(SMMCG.TrayIconVisible, true);

        public static bool TelemetryData => SMMI.GeneralSettingManager.GetSetting(SMMCG.TelemetryData, true);

        public static bool ExceptionData => SMMI.GeneralSettingManager.GetSetting(SMMCG.ExceptionData, true);

        public static bool AppExit => SMMI.GeneralSettingManager.GetSetting(SMMCG.AppExit, false);
    }
}