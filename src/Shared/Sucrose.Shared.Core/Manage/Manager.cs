using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSCEUCT = Sucrose.Shared.Core.Enum.UpdateChannelType;
using SSCEUET = Sucrose.Shared.Core.Enum.UpdateExtensionType;

namespace Sucrose.Shared.Core.Manage
{
    internal static class Manager
    {
        public static SSCEUET UpdateExtensionType => SMMI.UpdateSettingManager.GetSetting(SMC.UpdateExtensionType, SSCEUET.Executable);

        public static SSCEUCT UpdateChannelType => SMMI.UpdateSettingManager.GetSetting(SMC.UpdateChannelType, SSCEUCT.Release);
    }
}