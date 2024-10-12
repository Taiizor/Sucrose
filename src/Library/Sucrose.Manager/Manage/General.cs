using SHC = Skylark.Helper.Culture;
using SHS = Skylark.Helper.Skymath;
using SMMCG = Sucrose.Memory.Manage.Constant.General;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;

namespace Sucrose.Manager.Manage
{
    public static class General
    {
        public static string Culture => SMMI.GeneralSettingManager.GetSetting(SMMCG.Culture, SHC.CurrentUITwoLetterISOLanguageName);

        public static int Startup => SHS.Clamp(SMMI.GeneralSettingManager.GetSettingStable(SMMCG.Startup, 0), 0, 10);

        public static string UserAgent => SMMI.GeneralSettingManager.GetSetting(SMMCG.UserAgent, SMMRG.UserAgent);

        public static bool Statistics => SMMI.GeneralSettingManager.GetSetting(SMMCG.Statistics, true);

        public static bool AppExit => SMMI.GeneralSettingManager.GetSetting(SMMCG.AppExit, false);

        public static bool Visible => SMMI.GeneralSettingManager.GetSetting(SMMCG.Visible, true);

        public static bool Report => SMMI.GeneralSettingManager.GetSetting(SMMCG.Report, true);
    }
}