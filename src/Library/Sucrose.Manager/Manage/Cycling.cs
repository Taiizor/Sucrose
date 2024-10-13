using SHS = Skylark.Helper.Skymath;
using SMMCC = Sucrose.Memory.Manage.Constant.Cycling;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Manager.Manage
{
    public static class Cycling
    {
        public static int TransitionTime => SHS.Clamp(SMMI.CyclingSettingManager.GetSettingStable(SMMCC.TransitionTime, 30), 1, 999);

        public static int PassingTime => SHS.Clamp(SMMI.CyclingSettingManager.GetSettingStable(SMMCC.PassingTime, 0), 0, 99999);

        public static List<string> Exclusion => SMMI.CyclingSettingManager.GetSetting(SMMCC.Exclusion, new List<string>());

        public static bool Active => SMMI.CyclingSettingManager.GetSetting(SMMCC.Active, false);
    }
}