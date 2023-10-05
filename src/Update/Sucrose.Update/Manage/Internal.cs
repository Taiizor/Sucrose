using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSCEUT = Sucrose.Shared.Core.Enum.UpdateType;

namespace Sucrose.Update.Manage
{
    internal static class Internal
    {
        public static string Source = string.Empty;

        public static Mutex Mutex = new(true, SMR.UpdateMutex);

        public static readonly SSCEUT UpdateType = SMMI.UpdateSettingManager.GetSetting(SMC.UpdateType, SSCEUT.Compressed);
    }
}