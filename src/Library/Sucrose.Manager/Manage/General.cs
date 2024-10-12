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
using SMMCG = Sucrose.Memory.Manage.Constant.General;

namespace Sucrose.Manager.Manage
{
    public static class General
    {
        public static string Culture => SMMI.GeneralSettingManager.GetSetting(SMMCG.Culture, SHC.CurrentUITwoLetterISOLanguageName);

        public static int Startup => SHS.Clamp(SMMI.GeneralSettingManager.GetSettingStable(SMMCG.Startup, 0), 0, 10);

        public static string UserAgent => SMMI.GeneralSettingManager.GetSetting(SMMCG.UserAgent, SMR.UserAgent);

        public static bool Statistics => SMMI.GeneralSettingManager.GetSetting(SMMCG.Statistics, true);

        public static bool Report => SMMI.GeneralSettingManager.GetSetting(SMMCG.Report, true);
    }
}