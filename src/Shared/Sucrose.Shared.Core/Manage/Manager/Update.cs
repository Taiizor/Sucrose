using SMMCU = Sucrose.Memory.Manage.Constant.Update;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSCEUCT = Sucrose.Shared.Core.Enum.UpdateChannelType;
using SSCEUET = Sucrose.Shared.Core.Enum.UpdateExtensionType;

namespace Sucrose.Shared.Core.Manage.Manager
{
    internal static class Update
    {
        public static SSCEUET UpdateExtensionType => SMMI.UpdateSettingManager.GetSetting(SMMCU.UpdateExtensionType, SSCEUET.Executable);

        public static SSCEUCT UpdateChannelType => SMMI.UpdateSettingManager.GetSetting(SMMCU.UpdateChannelType, SSCEUCT.Release);
    }
}