using SMMCU = Sucrose.Memory.Manage.Constant.Update;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSCEUCT = Sucrose.Shared.Core.Enum.UpdateChannelType;
using SSCEUET = Sucrose.Shared.Core.Enum.UpdateExtensionType;

namespace Sucrose.Shared.Core.Manage.Manager
{
    internal static class Update
    {
        public static SSCEUET ExtensionType => SMMI.UpdateSettingManager.GetSetting(SMMCU.ExtensionType, SSCEUET.Executable);

        public static SSCEUCT ChannelType => SMMI.UpdateSettingManager.GetSetting(SMMCU.ChannelType, SSCEUCT.Release);
    }
}