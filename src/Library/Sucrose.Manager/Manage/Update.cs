using SESET = Skylark.Enum.StorageType;
using SHS = Skylark.Helper.Skymath;
using SMMCUE = Sucrose.Memory.Manage.Constant.Update;
using SMMI = Sucrose.Manager.Manage.Internal;

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