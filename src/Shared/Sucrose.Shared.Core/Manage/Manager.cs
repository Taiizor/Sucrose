using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSCECT = Sucrose.Shared.Core.Enum.ChannelType;
using SSCEUT = Sucrose.Shared.Core.Enum.UpdateType;

namespace Sucrose.Shared.Core.Manage
{
    internal static class Manager
    {
        public static SSCEUT UpdateType => SMMI.UpdateSettingManager.GetSetting(SMC.UpdateType, SSCEUT.Compressed);

        public static SSCECT ChannelType => SMMI.UpdateSettingManager.GetSetting(SMC.ChannelType, SSCECT.Release);
    }
}