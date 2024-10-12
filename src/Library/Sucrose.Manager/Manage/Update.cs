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
using SMMCUE = Sucrose.Memory.Manage.Constant.Update;
using SMMCB = Sucrose.Memory.Manage.Constant.Backgroundog;

namespace Sucrose.Manager.Manage
{
    public static class Update
    {
        public static int UpdateLimitValue => SHS.Clamp(SMMI.UpdateSettingManager.GetSettingStable(SMMCUE.UpdateLimitValue, 500), 0, 99999999);

        public static SESET UpdateLimitType => SMMI.UpdateSettingManager.GetSetting(SMMCUE.UpdateLimitType, SESET.Megabyte);

        public static DateTime UpdateTime => SMMI.UpdateSettingManager.GetSetting(SMMCUE.UpdateTime, new DateTime());

        public static bool UpdateState => SMMI.UpdateSettingManager.GetSetting(SMMCUE.UpdateState, false);

        public static bool AutoUpdate => SMMI.UpdateSettingManager.GetSetting(SMMCUE.AutoUpdate, true);
    }
}