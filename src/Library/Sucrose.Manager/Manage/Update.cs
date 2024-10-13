using SESET = Skylark.Enum.StorageType;
using SHS = Skylark.Helper.Skymath;
using SMMCU = Sucrose.Memory.Manage.Constant.Update;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Manager.Manage
{
    public static class Update
    {
        public static int LimitValue => SHS.Clamp(SMMI.UpdateSettingManager.GetSettingStable(SMMCU.LimitValue, 500), 0, 99999999);

        public static SESET LimitType => SMMI.UpdateSettingManager.GetSetting(SMMCU.LimitType, SESET.Megabyte);

        public static DateTime Time => SMMI.UpdateSettingManager.GetSetting(SMMCU.Time, new DateTime());

        public static bool State => SMMI.UpdateSettingManager.GetSetting(SMMCU.State, false);

        public static bool Auto => SMMI.UpdateSettingManager.GetSetting(SMMCU.Auto, true);
    }
}