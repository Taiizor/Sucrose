using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMMCL = Sucrose.Memory.Manage.Constant.Library;

namespace Sucrose.Manager.Manage
{
    public static class Library
    {
        public static string LibraryLocation => SMMI.LibrarySettingManager.GetSetting(SMMCL.LibraryLocation, Path.Combine(SMMRP.ApplicationData, SMR.AppName, SMR.Library));

        public static string LibrarySelected => SMMI.LibrarySettingManager.GetSetting(SMMCL.LibrarySelected, string.Empty);

        public static bool LibraryConfirm => SMMI.LibrarySettingManager.GetSetting(SMMCL.LibraryConfirm, true);

        public static bool LibraryDelete => SMMI.LibrarySettingManager.GetSetting(SMMCL.LibraryDelete, false);

        public static bool LibraryMove => SMMI.LibrarySettingManager.GetSetting(SMMCL.LibraryMove, true);
    }
}