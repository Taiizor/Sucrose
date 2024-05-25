using SHG = Skylark.Helper.Guidly;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Shared.Space.Helper
{
    internal static class User
    {
        public static Guid GetGuid()
        {
            return SMMM.Guid;
        }

        public static Guid NewGuid()
        {
            return Guid.NewGuid();
        }

        public static bool CheckGuid()
        {
            return !SMR.Guid.Equals(SHG.GuidToText(GetGuid()));
        }

        public static void ControlGuid()
        {
            if (!CheckGuid())
            {
                RegenerateGuid();
            }
        }

        public static void RegenerateGuid()
        {
            SMMI.UserSettingManager.SetSetting(SMC.Guid, NewGuid());
        }
    }
}