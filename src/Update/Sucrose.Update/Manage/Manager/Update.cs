using SMMCU = Sucrose.Memory.Manage.Constant.Update;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSCEUET = Sucrose.Shared.Core.Enum.UpdateExtensionType;
using SSDEUMT = Sucrose.Shared.Dependency.Enum.UpdateModuleType;

namespace Sucrose.Update.Manage.Manager
{
    internal static class Update
    {
        public static readonly SSCEUET UpdateExtensionType = SMMI.UpdateSettingManager.GetSetting(SMMCU.UpdateExtensionType, SSCEUET.Executable);

        public static readonly SSDEUMT UpdateModuleType = SMMI.UpdateSettingManager.GetSetting(SMMCU.UpdateModuleType, SSDEUMT.Downloader);
    }
}