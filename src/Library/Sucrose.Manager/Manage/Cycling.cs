using SHS = Skylark.Helper.Skymath;
using SMMCC = Sucrose.Memory.Manage.Constant.Cycling;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Manager.Manage
{
    public static class Cycling
    {
        public static int PassingCycyling => SHS.Clamp(SMMI.CyclingSettingManager.GetSettingStable(SMMCC.PassingCycyling, 0), 0, 99999);

        public static List<string> DisableCycyling => SMMI.CyclingSettingManager.GetSetting(SMMCC.DisableCycyling, new List<string>());

        public static int CycylingTime => SHS.Clamp(SMMI.CyclingSettingManager.GetSettingStable(SMMCC.CycylingTime, 30), 1, 999);

        public static bool Cycyling => SMMI.CyclingSettingManager.GetSetting(SMMCC.Cycyling, false);
    }
}