using SMMCA = Sucrose.Memory.Manage.Constant.Aurora;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Manager.Manage
{
    public static class Aurora
    {
        public static string AppProcessName => SMMI.AuroraSettingManager.GetSetting(SMMCA.AppProcessName, string.Empty);
    }
}