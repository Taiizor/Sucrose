using SMMCU = Sucrose.Memory.Manage.Constant.Update;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSDEUMT = Sucrose.Shared.Dependency.Enum.UpdateModuleType;
using SSDEUST = Sucrose.Shared.Dependency.Enum.UpdateServerType;

namespace Sucrose.Shared.Dependency.Manage.Manager
{
    internal static class Update
    {
        public static SSDEUMT UpdateModuleType => SMMI.UpdateSettingManager.GetSetting(SMMCU.UpdateModuleType, SSDEUMT.Downloader);

        public static SSDEUST UpdateServerType => SMMI.UpdateSettingManager.GetSetting(SMMCU.UpdateServerType, SSDEUST.Soferity);
    }
}