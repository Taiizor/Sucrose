using SMMCC = Sucrose.Memory.Manage.Constant.Cycling;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSDETCT = Sucrose.Shared.Dependency.Enum.TransitionCycleType;

namespace Sucrose.Shared.Dependency.Manage.Manager
{
    internal static class Cycling
    {
        public static SSDETCT TransitionType => SMMI.CyclingSettingManager.GetSetting(SMMCC.TransitionType, SSDETCT.Random);
    }
}